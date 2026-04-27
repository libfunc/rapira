# Forward-совместимость для RPC через обмен схемой

## Контекст

В rapira уже есть **частичная backward-совместимость** через `#[rapira(version = N)]`
и `#[rapira(since = M)]` — новый код может читать старые данные, отсутствующие
поля заполняются `Default::default()`.

Чего не хватает: **forward-совместимости** — старый код должен корректно читать
данные, сериализованные новой версией кода (с дополнительными полями).

Сценарии использования rapira:

- **БД** — есть миграции, текущая backward-совместимость покрывает все нужды.
- **RPC** — клиент-серверная архитектура, версии клиента и сервера могут
  расходиться в обе стороны. Здесь нужна полноценная двусторонняя
  совместимость.

## Ограничение

Текущий бинарный формат — плоская последовательность байт без префиксов длины,
маркеров полей и версии в данных. У старого получателя физически нет информации,
чтобы определить наличие и размер неизвестных полей. Особенно остро это
проявляется в коллекциях: элементы `Vec<T>` лежат вплотную, и любые «лишние
хвосты» сдвигают парсинг следующего элемента.

Прямого способа добиться forward-совместимости **без изменения формата нет**.
Однако в RPC-сценарии можно вынести описание формата за пределы данных — в
схему, которой стороны обмениваются один раз при коннекте.

## Идея

Бинарный формат данных не меняется. Появляется **runtime-схема** — описание
типов, которое:

1. Знает, как пройти курсором мимо значения любого типа без типизации
   (skip-функция).
2. Перечисляет поля структуры в порядке сериализации с именами и типами.

Клиент и сервер обмениваются схемами один раз при коннекте (или знают версию
протокола заранее и грузят соответствующую схему из встроенных). Дальше:

- сериализация — в формате отправителя, без изменений;
- десериализация — с учётом схемы отправителя: известные поля читаются,
  неизвестные пропускаются по их типу из схемы.

## Представление схемы

```rust
pub enum TypeSchema {
    Static(StaticKind),       // U8/U16/.../F64/Bool/Char/Duration/...
    String,
    Bytes,
    Option(Box<TypeSchema>),
    Vec(Box<TypeSchema>),
    Array(Box<TypeSchema>, u32),
    Tuple(Vec<TypeSchema>),
    Map { key: Box<TypeSchema>, value: Box<TypeSchema> },
    Struct(StructSchema),
    Enum(EnumSchema),
    Ref(TypeId),              // ссылка на тип в registry — для рекурсий и дедупликации
}

pub struct StructSchema {
    pub name: Cow<'static, str>,
    pub fields: Vec<FieldSchema>,
}

pub struct FieldSchema {
    pub name: Cow<'static, str>,
    pub ty: TypeSchema,
    pub since: Option<u8>,
}

pub struct EnumSchema {
    pub name: Cow<'static, str>,
    pub primitive: PrimKind,            // u8/u16/u32 — как сейчас
    pub variants: Vec<VariantSchema>,
}

pub struct VariantSchema {
    pub name: Cow<'static, str>,
    pub idx: u32,
    pub payload: VariantPayload,        // Unit | Tuple(Vec<TypeSchema>) | Struct(Vec<FieldSchema>)
}
```

Сама схема сериализуется обычным `Rapira` — отдельный wire-формат не нужен.
Чтобы избежать раздувания при рекурсивных и повторяющихся типах, поверх
делается **registry**: список уникальных типов + ссылки `Ref(id)`. Корневой
тип RPC-сообщения — индекс в registry.

## Обмен схемой

Один раз на коннект:

```
Client ──hello{client_schema_hash}──► Server
Client ◄─schema_registry{server}──── Server     // если хеш не совпал
Client ──schema_registry{client}───► Server     // если сервер не знает клиента
```

Дальше каждый держит у себя `PeerSchema { hash → registry }`. На каждый
RPC-вызов в шапке — один `u64` hash (или короткий ID), чтобы знать, по какой
версии схемы интерпретировать body. Per-message overhead — ноль.

Альтернатива: версия протокола передаётся числом, обе стороны при сборке кладут
свои схемы для каждой версии в бинарь. Тогда обмен не нужен — нужна только
версия.

## Алгоритм десериализации с peer-схемой

```rust
fn from_slice_with_schema<T: Rapira>(
    slice: &mut &[u8],
    peer: &StructSchema,
) -> Result<T>
```

Поля читаются в порядке peer-а (это и есть порядок байт):

```text
my := T::schema()                           // свой StructSchema, кэш в derive
my_by_name := index by field.name

for field in peer.fields:
    if let Some(my_field) = my_by_name.get(&field.name):
        if my_field.ty.compatible_with(&field.ty):
            value := <my_field.ty as Rapira>::from_slice(slice)?
            assign(my_field, value)
        else:
            return Err(SchemaMismatch)
    else:
        skip_value(slice, &field.ty)?

for my_field in my.fields not seen in peer:
    assign(my_field, Default::default())
```

`skip_value` — интерпретатор-«перематыватель»:

```text
fn skip_value(slice: &mut &[u8], ty: &TypeSchema) -> Result<()> {
    match ty {
        Static(k)        => advance(slice, k.size()),
        String | Bytes   => { let n = u32::from_slice(slice)?; advance(slice, n) }
        Option(inner)    => if read_byte(slice)? == 1 { skip_value(slice, inner)? }
        Vec(inner)       => for _ in 0..read_u32(slice)? { skip_value(slice, inner)? }
        Array(inner, n)  => for _ in 0..n { skip_value(slice, inner)? }
        Tuple(items)     => for t in items { skip_value(slice, t)? }
        Map{k, v}        => for _ in 0..read_u32(slice)? {
                                skip_value(slice, k)?;
                                skip_value(slice, v)?;
                            }
        Struct(s)        => for f in &s.fields { skip_value(slice, &f.ty)? }
        Enum(e)          => { let tag = read_prim(slice, e.primitive)?;
                              dispatch e.variants[tag] payload skip }
        Ref(id)          => skip_value(slice, registry.lookup(id)),
    }
}
```

Всё. ~100–200 строк интерпретатора.

## Что добавляется в rapira

1. **Модуль `schema`** с типами выше — чистые данные, ~300 строк.
2. **Trait `HasSchema { fn schema() -> &'static TypeSchema }`** — реализуется
   derive-ом и для всех встроенных типов.
3. **Derive-расширение**: для struct/enum генерируется
   `static SCHEMA: LazyLock<TypeSchema>`. Имена полей и типы у derive уже на
   руках — код почти зеркальный текущему `from_slice`. Атрибут
   `#[rapira(rename = "..")]` на поле — стандартное добавление.
4. **`skip_value`** — интерпретатор по схеме, ~150 строк.
5. **`from_slice_with_peer<T>(slice, peer_schema) -> Result<T>`** на трейте
   `Rapira` (метод по умолчанию делегирует в `from_slice`; derive генерирует
   версию для struct/enum).
6. **Schema registry + hash** для протокола обмена — ~200 строк, выносится в
   отдельный крейт `rapira-rpc`, чтобы не тащить в ядро.

Никаких изменений в `convert_to_bytes`/`size`/wire-format. БД и существующие
потребители не трогаются. Фича включается только при вызове
`from_slice_with_peer`.

## Подводные камни

- **Перестановки полей.** Если старый клиент послал A,B,C, а новый сервер ждёт
  C,B,A — match по имени с чтением «в порядке отправителя» работает. Главное —
  не перепутать «порядок чтения» (всегда peer-а) с «порядком сборки
  структуры».
- **Изменение типа поля.** `compatible_with` решает, считать ли `u32 → u64`
  допустимым. Безопаснее: только точное совпадение типа, иначе
  `SchemaMismatch`. Расширения — только новые поля.
- **Удаление поля.** Старый отправитель шлёт поле, у нового получателя его нет
  — корректно пропускается. Новый отправитель не шлёт поле, у старого
  получателя оно есть — `Default`. Семантика та же, что у текущего `since`.
- **Enum-варианты.** Если отправитель прислал неизвестный вариант — ошибка.
  Лечится политикой «append-only» и опциональным
  `Unknown`-вариантом у получателя. Это политика, не формат.
- **Производительность.** Совпадающие схемы (hash идентичен) — fast path,
  обычный `from_slice`, ноль накладных. Расходящиеся — интерпретатор по схеме,
  в разы медленнее, но сопоставимо с serde-форматами и редкий случай
  (обновление одной из сторон).
- **Размер схемы.** Через registry + Ref/hash экономно: пара КБ на средний
  RPC-API, отправляется один раз.

## Оценка трудоёмкости

- Типы схемы + сериализация + registry: 1–2 дня.
- Derive `HasSchema` для struct и обеих категорий enum: 1–2 дня.
- `skip_value` + тесты на все встроенные типы: 1 день.
- `from_slice_with_peer` в derive + интеграционные тесты «новый отправитель /
  старый получатель» и наоборот: 2–3 дня.
- Слой обмена/кеширования схем (`rapira-rpc`): 2–3 дня.

Итого ~7–11 рабочих дней на полную фичу с тестами. Ядро без RPC-слоя — 5–7
дней. Сложность не «архитектурная» — все изменения аддитивны, формата не
трогают, риск ломки нулевой.

## План внедрения

1. **Этап 1 — ядро.** Trait `HasSchema` + derive + `skip_value` +
   `from_slice_with_peer`. Самостоятельная и тестируемая часть, пригодится и
   вне RPC (миграции схем в логах, отладка дампов).
2. **Этап 2 — транспорт.** Отдельный крейт `rapira-rpc` для
   negotiation/registry/hash. Реализация зависит от транспорта (tcp+framing,
   QUIC, gRPC-style) и логично выносится отдельно.
