use super::ffi::AsPtr;
use super::{Conversation, Plugin};
use crate::PurpleMessageFlags;
use std::ffi::CString;
use std::os::raw::c_void;
use std::ptr::NonNull;

pub mod connections;
pub mod protocol_data;

pub use self::connections::Connections;
pub use self::protocol_data::Handle;

#[derive(Clone, Copy)]
pub struct Connection(NonNull<purple_sys::PurpleConnection>);

impl Connection {
    pub unsafe fn from_raw(ptr: *mut purple_sys::PurpleConnection) -> Option<Self> {
        NonNull::new(ptr).map(Self)
    }

    pub fn get_protocol_plugin(self) -> Option<Plugin> {
        let plugin_ptr = unsafe { purple_sys::purple_connection_get_prpl(self.0.as_ptr()) };
        if plugin_ptr.is_null() {
            None
        } else {
            Some(unsafe { Plugin::from_raw(plugin_ptr) })
        }
    }

    pub fn set_protocol_data(self, data: *mut c_void) {
        unsafe { purple_sys::purple_connection_set_protocol_data(self.0.as_ptr(), data) };
    }

    pub fn get_protocol_data(self) -> *mut c_void {
        unsafe { purple_sys::purple_connection_get_protocol_data(self.0.as_ptr()) }
    }

    pub fn get_account(self) -> crate::Account {
        unsafe {
            crate::Account::from_raw(purple_sys::purple_connection_get_account(self.0.as_ptr()))
        }
    }

    pub fn set_state(self, state: crate::PurpleConnectionState) {
        log::info!("Connection state: {:?}", state.0);
        unsafe { purple_sys::purple_connection_set_state(self.0.as_ptr(), state) };
    }

    pub fn error_reason(self, reason: crate::PurpleConnectionError, description: &str) {
        let c_description = CString::new(description).unwrap();
        unsafe {
            purple_sys::purple_connection_error_reason(
                self.0.as_ptr(),
                reason,
                c_description.as_ptr(),
            );
        }
    }

    pub fn serv_got_chat_in(self, sn: &str, who: &str, message: &str, mtime: i64) {
        unsafe {
            let c_sn = CString::new(sn).unwrap();
            let sn_hash = glib_sys::g_str_hash(c_sn.as_ptr() as *mut c_void);
            let c_who = CString::new(who).unwrap();
            let c_message = CString::new(message).unwrap();

            purple_sys::serv_got_chat_in(
                self.0.as_ptr(),
                sn_hash as i32,
                c_who.as_ptr(),
                PurpleMessageFlags::PURPLE_MESSAGE_RECV,
                c_message.as_ptr(),
                mtime,
            )
        }
    }

    pub fn serv_got_joined_chat(self, name: &str) -> Option<Conversation> {
        unsafe {
            let c_name = CString::new(name).unwrap();
            let name_hash = glib_sys::g_str_hash(c_name.as_ptr() as *mut c_void);
            let conv = purple_sys::serv_got_joined_chat(
                self.0.as_ptr(),
                name_hash as i32,
                c_name.as_ptr(),
            );
            Conversation::from_ptr(conv)
        }
    }
}

impl AsPtr for Connection {
    type PtrType = purple_sys::PurpleConnection;
    fn as_ptr(&self) -> *const purple_sys::PurpleConnection {
        self.0.as_ptr()
    }
}
