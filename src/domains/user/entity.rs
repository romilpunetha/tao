// Autogenerated by Thrift Compiler (0.22.0)
// DO NOT EDIT UNLESS YOU ARE SURE THAT YOU KNOW WHAT YOU ARE DOING

#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_extern_crates)]
#![allow(clippy::too_many_arguments, clippy::type_complexity, clippy::vec_box, clippy::wrong_self_convention)]
#![cfg_attr(rustfmt, rustfmt_skip)]

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::convert::{From, TryFrom};
use std::default::Default;
use std::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::rc::Rc;

use thrift::OrderedFloat;
use thrift::{ApplicationError, ApplicationErrorKind, ProtocolError, ProtocolErrorKind, TThriftClient};
use thrift::protocol::{TFieldIdentifier, TListIdentifier, TMapIdentifier, TMessageIdentifier, TMessageType, TInputProtocol, TOutputProtocol, TSerializable, TSetIdentifier, TStructIdentifier, TType};
use thrift::protocol::field_id;
use thrift::protocol::verify_expected_message_type;
use thrift::protocol::verify_expected_sequence_number;
use thrift::protocol::verify_expected_service_call;
use thrift::protocol::verify_required_field_exists;
use thrift::server::TProcessor;

pub type ENTUSER_USERNAME = String;

pub type ENTUSER_EMAIL = String;

pub type ENTUSER_FULL_NAME = String;

pub type ENTUSER_BIO = String;

pub type ENTUSER_PROFILE_PICTURE_URL = String;

pub type ENTUSER_LOCATION = String;

//
// ValidationException
//

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ValidationException {
  pub message: String,
  pub field: Option<String>,
}

impl ValidationException {
  pub fn new<F2>(message: String, field: F2) -> ValidationException where F2: Into<Option<String>> {
    ValidationException {
      message,
      field: field.into(),
    }
  }
}

impl TSerializable for ValidationException {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<ValidationException> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<String> = None;
    let mut f_2: Option<String> = None;
    loop {
      let field_ident = i_prot.read_field_begin()?;
      if field_ident.field_type == TType::Stop {
        break;
      }
      let field_id = field_id(&field_ident)?;
      match field_id {
        1 => {
          let val = i_prot.read_string()?;
          f_1 = Some(val);
        },
        2 => {
          let val = i_prot.read_string()?;
          f_2 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    verify_required_field_exists("ValidationException.message", &f_1)?;
    let ret = ValidationException {
      message: f_1.expect("auto-generated code should have checked for presence of required fields"),
      field: f_2,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("ValidationException");
    o_prot.write_struct_begin(&struct_ident)?;
    o_prot.write_field_begin(&TFieldIdentifier::new("message", TType::String, 1))?;
    o_prot.write_string(&self.message)?;
    o_prot.write_field_end()?;
    if let Some(ref fld_var) = self.field {
      o_prot.write_field_begin(&TFieldIdentifier::new("field", TType::String, 2))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}

impl Error for ValidationException {}

impl From<ValidationException> for thrift::Error {
  fn from(e: ValidationException) -> Self {
    thrift::Error::User(Box::new(e))
  }
}

impl Display for ValidationException {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "remote service threw ValidationException")
  }
}

//
// EntUser
//

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct EntUser {
  pub id: i64,
  pub username: String,
  pub email: String,
  pub created_time: i64,
  pub full_name: Option<String>,
  pub bio: Option<String>,
  pub profile_picture_url: Option<String>,
  pub last_active_time: Option<i64>,
  pub is_verified: bool,
  pub location: Option<String>,
  pub privacy_settings: Option<String>,
}

impl EntUser {
  pub fn new<F5, F6, F7, F8, F10, F11>(id: i64, username: String, email: String, created_time: i64, full_name: F5, bio: F6, profile_picture_url: F7, last_active_time: F8, is_verified: bool, location: F10, privacy_settings: F11) -> EntUser where F5: Into<Option<String>>, F6: Into<Option<String>>, F7: Into<Option<String>>, F8: Into<Option<i64>>, F10: Into<Option<String>>, F11: Into<Option<String>> {
    EntUser {
      id,
      username,
      email,
      created_time,
      full_name: full_name.into(),
      bio: bio.into(),
      profile_picture_url: profile_picture_url.into(),
      last_active_time: last_active_time.into(),
      is_verified,
      location: location.into(),
      privacy_settings: privacy_settings.into(),
    }
  }
}

impl TSerializable for EntUser {
  fn read_from_in_protocol(i_prot: &mut dyn TInputProtocol) -> thrift::Result<EntUser> {
    i_prot.read_struct_begin()?;
    let mut f_1: Option<i64> = None;
    let mut f_2: Option<String> = None;
    let mut f_3: Option<String> = None;
    let mut f_4: Option<i64> = None;
    let mut f_5: Option<String> = None;
    let mut f_6: Option<String> = None;
    let mut f_7: Option<String> = None;
    let mut f_8: Option<i64> = None;
    let mut f_9: Option<bool> = None;
    let mut f_10: Option<String> = None;
    let mut f_11: Option<String> = None;
    loop {
      let field_ident = i_prot.read_field_begin()?;
      if field_ident.field_type == TType::Stop {
        break;
      }
      let field_id = field_id(&field_ident)?;
      match field_id {
        1 => {
          let val = i_prot.read_i64()?;
          f_1 = Some(val);
        },
        2 => {
          let val = i_prot.read_string()?;
          f_2 = Some(val);
        },
        3 => {
          let val = i_prot.read_string()?;
          f_3 = Some(val);
        },
        4 => {
          let val = i_prot.read_i64()?;
          f_4 = Some(val);
        },
        5 => {
          let val = i_prot.read_string()?;
          f_5 = Some(val);
        },
        6 => {
          let val = i_prot.read_string()?;
          f_6 = Some(val);
        },
        7 => {
          let val = i_prot.read_string()?;
          f_7 = Some(val);
        },
        8 => {
          let val = i_prot.read_i64()?;
          f_8 = Some(val);
        },
        9 => {
          let val = i_prot.read_bool()?;
          f_9 = Some(val);
        },
        10 => {
          let val = i_prot.read_string()?;
          f_10 = Some(val);
        },
        11 => {
          let val = i_prot.read_string()?;
          f_11 = Some(val);
        },
        _ => {
          i_prot.skip(field_ident.field_type)?;
        },
      };
      i_prot.read_field_end()?;
    }
    i_prot.read_struct_end()?;
    verify_required_field_exists("EntUser.id", &f_1)?;
    verify_required_field_exists("EntUser.username", &f_2)?;
    verify_required_field_exists("EntUser.email", &f_3)?;
    verify_required_field_exists("EntUser.created_time", &f_4)?;
    verify_required_field_exists("EntUser.is_verified", &f_9)?;
    let ret = EntUser {
      id: f_1.expect("auto-generated code should have checked for presence of required fields"),
      username: f_2.expect("auto-generated code should have checked for presence of required fields"),
      email: f_3.expect("auto-generated code should have checked for presence of required fields"),
      created_time: f_4.expect("auto-generated code should have checked for presence of required fields"),
      full_name: f_5,
      bio: f_6,
      profile_picture_url: f_7,
      last_active_time: f_8,
      is_verified: f_9.expect("auto-generated code should have checked for presence of required fields"),
      location: f_10,
      privacy_settings: f_11,
    };
    Ok(ret)
  }
  fn write_to_out_protocol(&self, o_prot: &mut dyn TOutputProtocol) -> thrift::Result<()> {
    let struct_ident = TStructIdentifier::new("EntUser");
    o_prot.write_struct_begin(&struct_ident)?;
    o_prot.write_field_begin(&TFieldIdentifier::new("id", TType::I64, 1))?;
    o_prot.write_i64(self.id)?;
    o_prot.write_field_end()?;
    o_prot.write_field_begin(&TFieldIdentifier::new("username", TType::String, 2))?;
    o_prot.write_string(&self.username)?;
    o_prot.write_field_end()?;
    o_prot.write_field_begin(&TFieldIdentifier::new("email", TType::String, 3))?;
    o_prot.write_string(&self.email)?;
    o_prot.write_field_end()?;
    o_prot.write_field_begin(&TFieldIdentifier::new("created_time", TType::I64, 4))?;
    o_prot.write_i64(self.created_time)?;
    o_prot.write_field_end()?;
    if let Some(ref fld_var) = self.full_name {
      o_prot.write_field_begin(&TFieldIdentifier::new("full_name", TType::String, 5))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.bio {
      o_prot.write_field_begin(&TFieldIdentifier::new("bio", TType::String, 6))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.profile_picture_url {
      o_prot.write_field_begin(&TFieldIdentifier::new("profile_picture_url", TType::String, 7))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(fld_var) = self.last_active_time {
      o_prot.write_field_begin(&TFieldIdentifier::new("last_active_time", TType::I64, 8))?;
      o_prot.write_i64(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_begin(&TFieldIdentifier::new("is_verified", TType::Bool, 9))?;
    o_prot.write_bool(self.is_verified)?;
    o_prot.write_field_end()?;
    if let Some(ref fld_var) = self.location {
      o_prot.write_field_begin(&TFieldIdentifier::new("location", TType::String, 10))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    if let Some(ref fld_var) = self.privacy_settings {
      o_prot.write_field_begin(&TFieldIdentifier::new("privacy_settings", TType::String, 11))?;
      o_prot.write_string(fld_var)?;
      o_prot.write_field_end()?
    }
    o_prot.write_field_stop()?;
    o_prot.write_struct_end()
  }
}

