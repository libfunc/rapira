extern crate proc_macro2;
extern crate quote;
extern crate syn;

mod attributes;
mod enum_with_primitive;
mod enums;
mod simple_enum;
mod structs;
mod utils;

use enum_with_primitive::enum_with_primitive_serializer;
use enums::enum_serializer;
use simple_enum::simple_enum_serializer;
use structs::struct_serializer;
use syn::{parse_macro_input, Data, DeriveInput};

#[proc_macro_derive(Rapira, attributes(rapira))]
pub fn serializer_trait(stream: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = parse_macro_input!(stream as DeriveInput);
    let name = &ast.ident;
    let data = &ast.data;

    match data {
        Data::Struct(data_struct) => struct_serializer(data_struct, name),
        Data::Enum(data_enum) => {
            let is_simple_enum = data_enum.variants.iter().all(|item| item.fields.is_empty());
            if is_simple_enum {
                simple_enum_serializer(name)
            } else {
                let primitive_name = attributes::get_primitive_name(&ast.attrs);

                match primitive_name {
                    Some(primitive_name) => {
                        enum_with_primitive_serializer(data_enum, name, &primitive_name)
                    }
                    None => {
                        let skip_static_size = attributes::skip_static_size(&ast.attrs);
                        enum_serializer(data_enum, name, skip_static_size)
                    }
                }
            }
        }
        Data::Union(_) => {
            panic!("unions not supported, but Rust enums is implemented Rapira trait (use Enums instead)")
        }
    }
}
