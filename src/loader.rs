use super::handlers::entrypoints;
use crate::PrplPlugin;
use log::info;
use std::ffi::CString;
use std::marker::PhantomData;
use std::os::raw::c_void;

#[derive(Default)]
pub struct RegisterContext<P> {
    info: Box<purple_sys::PurplePluginInfo>,
    extra_info: Box<purple_sys::PurplePluginProtocolInfo>,
    _phantom_marker: PhantomData<P>,
}

impl<P> RegisterContext<P> {
    pub fn new() -> Self {
        RegisterContext {
            info: Box::new(purple_sys::PurplePluginInfo::default()),
            extra_info: Box::new(purple_sys::PurplePluginProtocolInfo::default()),
            _phantom_marker: PhantomData,
        }
    }
    pub fn into_raw(mut self) -> *mut purple_sys::PurplePluginInfo {
        self.extra_info.roomlist_get_list = Some(entrypoints::roomlist_get_list_handler);

        // If any protocol options have been set, they are back to front, so
        // reverse the list now
        unsafe {
            self.extra_info.protocol_options =
                glib_sys::g_list_reverse(self.extra_info.protocol_options);
        }

        self.info.extra_info = Box::into_raw(self.extra_info) as *mut c_void;

        Box::into_raw(self.info)
    }

    pub fn with_info(mut self, info: PrplInfo) -> Self {
        self.info.id = CString::new(info.id).unwrap().into_raw();
        self.info.name = CString::new(info.name).unwrap().into_raw();
        self.info.version = CString::new(info.version).unwrap().into_raw();
        self.info.summary = CString::new(info.summary).unwrap().into_raw();
        self.info.description = CString::new(info.description).unwrap().into_raw();
        self.info.author = CString::new(info.author).unwrap().into_raw();
        self.info.homepage = CString::new(info.homepage).unwrap().into_raw();
        self.info.actions = Some(entrypoints::actions);
        self
    }

    fn with_option(mut self, option: PrplOption) -> Self {
        let mut list: *mut glib_sys::GList = self.extra_info.protocol_options;
        unsafe {
            let ptr: *mut purple_sys::PurpleAccountOption = match option.def {
                PrplOptionValue::String(def) => purple_sys::purple_account_option_string_new(
                    CString::new(option.text).unwrap().into_raw(),
                    CString::new(option.key).unwrap().into_raw(),
                    CString::new(def).unwrap().into_raw(),
                ),
                PrplOptionValue::Bool(def) => purple_sys::purple_account_option_bool_new(
                    CString::new(option.text).unwrap().into_raw(),
                    CString::new(option.key).unwrap().into_raw(),
                    def.into(),
                ),
            };

            purple_sys::purple_account_option_set_masked(ptr, option.masked.into());

            list = glib_sys::g_list_prepend(list, glib::translate::Ptr::to(ptr));
        }

        self.extra_info.protocol_options = list;

        self
    }

    pub fn with_string_option(self, name: String, key: String, def: String) -> Self {
        self.with_option(PrplOption {
            text: name,
            key: key,
            def: PrplOptionValue::String(def),
            masked: false,
        })
    }

    pub fn with_password_option(self, name: String, key: String, def: String) -> Self {
        self.with_option(PrplOption {
            text: name,
            key: key,
            def: PrplOptionValue::String(def),
            masked: true,
        })
    }

    pub fn with_bool_option(self, name: String, key: String, def: bool) -> Self {
        self.with_option(PrplOption {
            text: name,
            key: key,
            def: PrplOptionValue::Bool(def),
            masked: false,
        })
    }
}

pub struct PrplPluginLoader<P: PrplPlugin>(*mut purple_sys::PurplePlugin, PhantomData<P>);

impl<P: PrplPlugin> PrplPluginLoader<P> {
    pub unsafe fn from_raw(ptr: *mut purple_sys::PurplePlugin) -> Self {
        Self(ptr, PhantomData)
    }

    pub fn init(&self) -> i32 {
        let prpl_plugin = Box::new(P::new());
        let register_context: RegisterContext<P::Plugin> = RegisterContext::new();
        let register_context = prpl_plugin.register(register_context);

        // Unsafe required to dereference the pointers and call
        // purple_plugin_register. Safe otherwise.
        unsafe {
            (*self.0).info = register_context.into_raw();
            (*self.0).extra = Box::into_raw(prpl_plugin) as *mut c_void;

            info!("Registering");
            purple_sys::purple_plugin_register(self.0)
        }
    }
}

pub struct PrplInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub summary: String,
    pub description: String,
    pub author: String,
    pub homepage: String,
}

enum PrplOptionValue {
    Bool(bool),
    String(String),
}

struct PrplOption {
    text: String,
    key: String,
    def: PrplOptionValue,
    masked: bool,
}

impl Default for PrplInfo {
    fn default() -> Self {
        PrplInfo {
            id: "".into(),
            name: "".into(),
            version: "".into(),
            summary: "".into(),
            description: "".into(),
            author: "".into(),
            homepage: "".into(),
        }
    }
}

macro_rules! impl_handler_builder {
    ($($f:ident => $t:ident)*) => ($(
        paste::item! {
            impl<T: crate::handlers::traits::[<$t>]> RegisterContext<T> {
                #[allow(dead_code)]
                pub fn [<enable_$f>](mut self) -> Self {
                    self.info.[<$f>] = Some(crate::handlers::entrypoints::[<$f>]::<T>);
                    self
                }
            }
        }
    )*)
}

macro_rules! impl_extra_handler_builder {
    ($($f:ident => $t:ident)*) => ($(
        paste::item! {
            impl<T: crate::handlers::traits::[<$t>]> RegisterContext<T> {
                #[allow(dead_code)]
                pub fn [<enable_$f>](mut self) -> Self {
                    self.extra_info.[<$f>] = Some(crate::handlers::entrypoints::[<$f>]::<T>);
                    self
                }
            }
        }
    )*)
}

impl_handler_builder! {
    load => LoadHandler
}

impl_extra_handler_builder! {
    login => LoginHandler
    chat_info => ChatInfoHandler
    chat_info_defaults => ChatInfoDefaultsHandler
    close => CloseHandler
    status_types => StatusTypeHandler
    list_icon => ListIconHandler
    join_chat => JoinChatHandler
    chat_leave => ChatLeaveHandler
    convo_closed => ConvoClosedHandler
    get_chat_name => GetChatNameHandler
    send_im => SendIMHandler
    chat_send => ChatSendHandler
    get_cb_alias => GetChatBuddyAlias
}
