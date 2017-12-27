// This file is generated. Do not edit
// @generated

// https://github.com/Manishearth/rust-clippy/issues/702
#![allow(unknown_lints)]
#![allow(clippy)]

#![cfg_attr(rustfmt, rustfmt_skip)]

#![allow(box_pointers)]
#![allow(dead_code)]
#![allow(missing_docs)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(trivial_casts)]
#![allow(unsafe_code)]
#![allow(unused_imports)]
#![allow(unused_results)]

use protobuf::Message as Message_imported_for_functions;
use protobuf::ProtobufEnum as ProtobufEnum_imported_for_functions;

#[derive(PartialEq,Clone,Default)]
pub struct TokenState {
    // message fields
    pub name: ::std::string::String,
    pub symbol: ::std::string::String,
    pub total_supply: u64,
    pub balance_of: ::std::collections::HashMap<::std::string::String, u64>,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for TokenState {}

impl TokenState {
    pub fn new() -> TokenState {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static TokenState {
        static mut instance: ::protobuf::lazy::Lazy<TokenState> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const TokenState,
        };
        unsafe {
            instance.get(TokenState::new)
        }
    }

    // string name = 1;

    pub fn clear_name(&mut self) {
        self.name.clear();
    }

    // Param is passed by value, moved
    pub fn set_name(&mut self, v: ::std::string::String) {
        self.name = v;
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_name(&mut self) -> &mut ::std::string::String {
        &mut self.name
    }

    // Take field
    pub fn take_name(&mut self) -> ::std::string::String {
        ::std::mem::replace(&mut self.name, ::std::string::String::new())
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    fn get_name_for_reflect(&self) -> &::std::string::String {
        &self.name
    }

    fn mut_name_for_reflect(&mut self) -> &mut ::std::string::String {
        &mut self.name
    }

    // string symbol = 2;

    pub fn clear_symbol(&mut self) {
        self.symbol.clear();
    }

    // Param is passed by value, moved
    pub fn set_symbol(&mut self, v: ::std::string::String) {
        self.symbol = v;
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_symbol(&mut self) -> &mut ::std::string::String {
        &mut self.symbol
    }

    // Take field
    pub fn take_symbol(&mut self) -> ::std::string::String {
        ::std::mem::replace(&mut self.symbol, ::std::string::String::new())
    }

    pub fn get_symbol(&self) -> &str {
        &self.symbol
    }

    fn get_symbol_for_reflect(&self) -> &::std::string::String {
        &self.symbol
    }

    fn mut_symbol_for_reflect(&mut self) -> &mut ::std::string::String {
        &mut self.symbol
    }

    // uint64 total_supply = 3;

    pub fn clear_total_supply(&mut self) {
        self.total_supply = 0;
    }

    // Param is passed by value, moved
    pub fn set_total_supply(&mut self, v: u64) {
        self.total_supply = v;
    }

    pub fn get_total_supply(&self) -> u64 {
        self.total_supply
    }

    fn get_total_supply_for_reflect(&self) -> &u64 {
        &self.total_supply
    }

    fn mut_total_supply_for_reflect(&mut self) -> &mut u64 {
        &mut self.total_supply
    }

    // repeated .token.TokenState.BalanceOfEntry balance_of = 4;

    pub fn clear_balance_of(&mut self) {
        self.balance_of.clear();
    }

    // Param is passed by value, moved
    pub fn set_balance_of(&mut self, v: ::std::collections::HashMap<::std::string::String, u64>) {
        self.balance_of = v;
    }

    // Mutable pointer to the field.
    pub fn mut_balance_of(&mut self) -> &mut ::std::collections::HashMap<::std::string::String, u64> {
        &mut self.balance_of
    }

    // Take field
    pub fn take_balance_of(&mut self) -> ::std::collections::HashMap<::std::string::String, u64> {
        ::std::mem::replace(&mut self.balance_of, ::std::collections::HashMap::new())
    }

    pub fn get_balance_of(&self) -> &::std::collections::HashMap<::std::string::String, u64> {
        &self.balance_of
    }

    fn get_balance_of_for_reflect(&self) -> &::std::collections::HashMap<::std::string::String, u64> {
        &self.balance_of
    }

    fn mut_balance_of_for_reflect(&mut self) -> &mut ::std::collections::HashMap<::std::string::String, u64> {
        &mut self.balance_of
    }
}

impl ::protobuf::Message for TokenState {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    ::protobuf::rt::read_singular_proto3_string_into(wire_type, is, &mut self.name)?;
                },
                2 => {
                    ::protobuf::rt::read_singular_proto3_string_into(wire_type, is, &mut self.symbol)?;
                },
                3 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_uint64()?;
                    self.total_supply = tmp;
                },
                4 => {
                    ::protobuf::rt::read_map_into::<::protobuf::types::ProtobufTypeString, ::protobuf::types::ProtobufTypeUint64>(wire_type, is, &mut self.balance_of)?;
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if !self.name.is_empty() {
            my_size += ::protobuf::rt::string_size(1, &self.name);
        }
        if !self.symbol.is_empty() {
            my_size += ::protobuf::rt::string_size(2, &self.symbol);
        }
        if self.total_supply != 0 {
            my_size += ::protobuf::rt::value_size(3, self.total_supply, ::protobuf::wire_format::WireTypeVarint);
        }
        my_size += ::protobuf::rt::compute_map_size::<::protobuf::types::ProtobufTypeString, ::protobuf::types::ProtobufTypeUint64>(4, &self.balance_of);
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if !self.name.is_empty() {
            os.write_string(1, &self.name)?;
        }
        if !self.symbol.is_empty() {
            os.write_string(2, &self.symbol)?;
        }
        if self.total_supply != 0 {
            os.write_uint64(3, self.total_supply)?;
        }
        ::protobuf::rt::write_map_with_cached_sizes::<::protobuf::types::ProtobufTypeString, ::protobuf::types::ProtobufTypeUint64>(4, &self.balance_of, os)?;
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for TokenState {
    fn new() -> TokenState {
        TokenState::new()
    }

    fn descriptor_static(_: ::std::option::Option<TokenState>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "name",
                    TokenState::get_name_for_reflect,
                    TokenState::mut_name_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "symbol",
                    TokenState::get_symbol_for_reflect,
                    TokenState::mut_symbol_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeUint64>(
                    "total_supply",
                    TokenState::get_total_supply_for_reflect,
                    TokenState::mut_total_supply_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_map_accessor::<_, ::protobuf::types::ProtobufTypeString, ::protobuf::types::ProtobufTypeUint64>(
                    "balance_of",
                    TokenState::get_balance_of_for_reflect,
                    TokenState::mut_balance_of_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<TokenState>(
                    "TokenState",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for TokenState {
    fn clear(&mut self) {
        self.clear_name();
        self.clear_symbol();
        self.clear_total_supply();
        self.clear_balance_of();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for TokenState {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for TokenState {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

static file_descriptor_proto_data: &'static [u8] = b"\
    \n\x11token_state.proto\x12\x05token\"\xda\x01\n\nTokenState\x12\x12\n\
    \x04name\x18\x01\x20\x01(\tR\x04name\x12\x16\n\x06symbol\x18\x02\x20\x01\
    (\tR\x06symbol\x12!\n\x0ctotal_supply\x18\x03\x20\x01(\x04R\x0btotalSupp\
    ly\x12?\n\nbalance_of\x18\x04\x20\x03(\x0b2\x20.token.TokenState.Balance\
    OfEntryR\tbalanceOf\x1a<\n\x0eBalanceOfEntry\x12\x10\n\x03key\x18\x01\
    \x20\x01(\tR\x03key\x12\x14\n\x05value\x18\x02\x20\x01(\x04R\x05value:\
    \x028\x01b\x06proto3\
";

static mut file_descriptor_proto_lazy: ::protobuf::lazy::Lazy<::protobuf::descriptor::FileDescriptorProto> = ::protobuf::lazy::Lazy {
    lock: ::protobuf::lazy::ONCE_INIT,
    ptr: 0 as *const ::protobuf::descriptor::FileDescriptorProto,
};

fn parse_descriptor_proto() -> ::protobuf::descriptor::FileDescriptorProto {
    ::protobuf::parse_from_bytes(file_descriptor_proto_data).unwrap()
}

pub fn file_descriptor_proto() -> &'static ::protobuf::descriptor::FileDescriptorProto {
    unsafe {
        file_descriptor_proto_lazy.get(|| {
            parse_descriptor_proto()
        })
    }
}
