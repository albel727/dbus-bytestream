//! Functions for creating and modifying messages to send across the message bus.
use std::ops::DerefMut;
use std::cell::RefCell;

use dbus_serialize::types::{Path,Variant,Value,BasicValue,Signature};

use marshal::{Marshal,pad_to_multiple};
use demarshal::{demarshal,DemarshalError};

#[derive(Default,PartialEq,Eq)]
pub struct MessageType(pub u8);

impl ::std::fmt::Debug for MessageType {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        let mut debug_trait_builder = f.debug_tuple("MessageType");
        match *self {
            MESSAGE_TYPE_INVALID => debug_trait_builder.field(&"INVALID"),
            MESSAGE_TYPE_METHOD_CALL => debug_trait_builder.field(&"METHOD_CALL"),
            MESSAGE_TYPE_METHOD_RETURN => debug_trait_builder.field(&"METHOD_RETURN"),
            MESSAGE_TYPE_ERROR => debug_trait_builder.field(&"ERROR"),
            MESSAGE_TYPE_SIGNAL => debug_trait_builder.field(&"SIGNAL"),
            MessageType(type_id) => debug_trait_builder.field(&type_id),
        };
        debug_trait_builder.finish()
    }
}

pub const MESSAGE_TYPE_INVALID : MessageType        = MessageType(0);
pub const MESSAGE_TYPE_METHOD_CALL : MessageType    = MessageType(1);
pub const MESSAGE_TYPE_METHOD_RETURN : MessageType  = MessageType(2);
pub const MESSAGE_TYPE_ERROR : MessageType          = MessageType(3);
pub const MESSAGE_TYPE_SIGNAL : MessageType         = MessageType(4);

pub const HEADER_FIELD_INVALID : u8     = 0;
pub const HEADER_FIELD_PATH: u8         = 1;
pub const HEADER_FIELD_INTERFACE: u8    = 2;
pub const HEADER_FIELD_MEMBER: u8       = 3;
pub const HEADER_FIELD_ERROR_NAME: u8   = 4;
pub const HEADER_FIELD_REPLY_SERIAL: u8 = 5;
pub const HEADER_FIELD_DESTINATION: u8  = 6;
pub const HEADER_FIELD_SENDER: u8       = 7;
pub const HEADER_FIELD_SIGNATURE: u8    = 8;

pub const FLAGS_NO_REPLY_EXPECTED : u8  = 1;

pub struct HeaderField (
    pub u8,
    pub Variant
);

impl ::std::fmt::Debug for HeaderField {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        let HeaderField(field_id, ref variant) = *self;
        let mut debug_trait_builder = f.debug_tuple("HeaderField");
        let desc = match field_id {
            HEADER_FIELD_INVALID => "INVALID",
            HEADER_FIELD_PATH => "PATH",
            HEADER_FIELD_INTERFACE => "INTERFACE",
            HEADER_FIELD_MEMBER => "MEMBER",
            HEADER_FIELD_ERROR_NAME => "ERROR_NAME",
            HEADER_FIELD_REPLY_SERIAL => "REPLY_SERIAL",
            HEADER_FIELD_DESTINATION => "DESTINATION",
            HEADER_FIELD_SENDER => "SENDER",
            HEADER_FIELD_SIGNATURE => "SIGNATURE",
            _ => "",
        };
        if !desc.is_empty() {
            debug_trait_builder.field(&desc);
        } else {
            debug_trait_builder.field(&field_id);
        }
        let _ = debug_trait_builder.field(variant);
        debug_trait_builder.finish()
    }
}

impl Marshal for HeaderField {
    fn dbus_encode(&self, buf: &mut Vec<u8>) -> usize {
        pad_to_multiple(buf, 8);
        let start_len = buf.len();
        let code = self.0 as u8;
        code.dbus_encode(buf);
        self.1.dbus_encode(buf);
        buf.len() - start_len
    }
    fn get_type(&self) -> String {
        "(yv)".to_owned()
    }
}

/// Represents a received message from the message bus
#[derive(Debug,Default)]
pub struct Message {
    pub big_endian: bool,
    pub message_type: MessageType,
    pub flags: u8,
    pub version: u8,
    pub serial: u32,
    pub headers: Vec<HeaderField>,
    pub body: Vec<u8>,

    body_cache: RefCell<Option<Result<Option<Vec<Value>>, DemarshalError>>>
}

impl Marshal for Message {
    fn dbus_encode (&self, buf: &mut Vec<u8>) -> usize {
        let endian = if self.big_endian { 'B' as u8 } else { 'l' as u8 };
        endian.dbus_encode(buf);
        self.message_type.0.dbus_encode(buf);
        self.flags.dbus_encode(buf);
        self.version.dbus_encode(buf);
        let len : u32 = self.body.len() as u32;
        len.dbus_encode(buf);
        self.serial.dbus_encode(buf);
        self.headers.dbus_encode(buf);
        pad_to_multiple(buf, 8);
        0
    }

    fn get_type (&self) -> String {
        panic!("Don't do that.")
    }
}

/// Create a Message for a D-Bus method call.  Once a Message is created, arguments
/// can be added with Message.add_arg
pub fn create_method_call (dest: &str, path: &str, iface: &str, method: &str) -> Message {
    Message {
        big_endian: false,
        message_type: MESSAGE_TYPE_METHOD_CALL,
        flags: 0,
        version: 1,
        serial: 0,
        headers: Vec::new(),
        body: Vec::new(),

        body_cache: RefCell::new(None),
    }.add_header(HEADER_FIELD_DESTINATION,
                 Variant::new(Value::from(dest), "s"))
     .add_header(HEADER_FIELD_PATH,
                 Variant::new(Value::BasicValue(BasicValue::ObjectPath(Path(path.to_owned()))), "o"))
     .add_header(HEADER_FIELD_INTERFACE,
                 Variant::new(Value::from(iface), "s"))
     .add_header(HEADER_FIELD_MEMBER,
                 Variant::new(Value::from(method), "s"))
}

/// Create a Message for a D-Bus method return.  Once created, return values can be added
/// with Message.add_arg
pub fn create_method_return(reply_serial: u32) -> Message {
    Message {
        big_endian: false,
        message_type: MESSAGE_TYPE_METHOD_RETURN,
        flags: 0,
        version: 1,
        serial: 0,
        headers: Vec::new(),
        body: Vec::new(),

        body_cache: RefCell::new(None),
    }.add_header(HEADER_FIELD_REPLY_SERIAL,
                 Variant::new(Value::from(reply_serial), "u"))
}

/// Create a Message for a D-Bus error.  Once created, return values can be added
/// with Message.add_arg
pub fn create_error(error_name: &str, reply_serial: u32) -> Message {
    Message {
        big_endian: false,
        message_type: MESSAGE_TYPE_ERROR,
        flags: 0,
        version: 1,
        serial: 0,
        headers: Vec::new(),
        body: Vec::new(),

        body_cache: RefCell::new(None),
    }.add_header(HEADER_FIELD_REPLY_SERIAL,
                 Variant::new(Value::from(reply_serial), "u"))
     .add_header(HEADER_FIELD_ERROR_NAME,
                 Variant::new(Value::from(error_name), "s"))
}

/// Create a Message for a D-Bus signal.  Once created, return values can be added
/// with Message.add_arg
pub fn create_signal(path: &str, interface: &str, member: &str) -> Message {
    Message {
        big_endian: false,
        message_type: MESSAGE_TYPE_SIGNAL,
        flags: 0,
        version: 1,
        serial: 0,
        headers: Vec::new(),
        body: Vec::new(),

        body_cache: RefCell::new(None),
    }.add_header(HEADER_FIELD_PATH,
                 Variant::new(Value::BasicValue(BasicValue::ObjectPath(Path(path.to_owned()))), "o"))
     .add_header(HEADER_FIELD_INTERFACE,
                 Variant::new(Value::from(interface), "s"))
     .add_header(HEADER_FIELD_MEMBER,
                 Variant::new(Value::from(member), "s"))
}

impl Message {
    /// Add the given argument to the Message.  Accepts anything that implements the Marshal
    /// trait, which is most basic types, as well as the general-purpose
    /// dbus_serialize::types::Value enum.
    ///
    /// Note that these calls can be chained together to add multiple arguments, see the example
    ///
    /// # Examples
    /// ```
    /// dbus_bytestream::message::create_method_call("foo", "/bar", "baz", "bloop")
    ///     .add_arg(&1)
    ///     .add_arg(&"string");
    /// ```
    pub fn add_arg(mut self, arg: &Marshal) -> Message {
        if let None = self.get_header(HEADER_FIELD_SIGNATURE) {
            let value = Value::BasicValue(BasicValue::Signature(Signature("".to_owned())));
            let variant = Variant::new(value, "g");
            self = self.add_header(HEADER_FIELD_SIGNATURE, variant);
        };
        {
            let b : &mut Box<Value> = &mut self.get_header_mut(HEADER_FIELD_SIGNATURE).unwrap().object;
            let val : &mut Value = b.deref_mut();
            match *val {
                Value::BasicValue(BasicValue::Signature(ref mut s)) => s.0.push_str(&arg.get_type()),
                _ => panic!("Garbage in signature field")
            };
        }
        arg.dbus_encode(&mut self.body);
        self
    }

    pub fn get_header(&self, name: u8) -> Option<&Variant> {
        self.headers.iter().position(|x| { x.0 == name })
            .map(|idx| &self.headers[idx].1)
    }

    pub fn get_header_mut(&mut self, name: u8) -> Option<&mut Variant> {
        match self.headers.iter().position(|x| { x.0 == name }) {
            Some(idx) => Some(&mut self.headers[idx].1),
            _ => None
        }
    }

    pub fn add_header(mut self, name: u8, val: Variant) -> Message {
        self.headers.push(HeaderField (name, val));
        self
    }

    /// Get the sequence of Values from out of a Message.  Returns None if the message doesn't have
    /// a body.
    pub fn get_body(&self) -> Result<Option<Vec<Value>>,DemarshalError> {
        if self.body.is_empty() {
            return Ok(None);
        }
        let cached = self.body_cache.borrow().is_some();
        if !cached {
            // Get the signature out of the headers
            let v = match self.headers.iter().position(|x| { x.0 == HEADER_FIELD_SIGNATURE }) {
                Some(idx) => &self.headers[idx].1,
                None => return Ok(None)
            };

            let sigval = match *v.object {
                Value::BasicValue(BasicValue::Signature(ref x)) => x,
                _ => return Ok(None)
            };

            let mut body = self.body.clone();
            let mut sig = "(".to_owned() + &sigval.0 + ")";
            let mut offset = 0;
            *self.body_cache.borrow_mut() = Some((|| {
                match try!(demarshal(&mut body, &mut offset, &mut sig)) {
                    Value::Struct(x) => Ok(Some(x.objects)),
                    x => panic!("Didn't get a struct: {:?}", x)
                }
            })());
        }
        self.body_cache.borrow().as_ref().unwrap().clone()
    }
}

#[test]
fn test_msg () {
    create_method_call("foo", "bar", "baz", "floob")
        .add_arg(&1)
        .add_arg(&2);
}
