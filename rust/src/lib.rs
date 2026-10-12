//! YTHome: a launcher icon that opens m.youtube.com in a Chrome Custom Tab.
//!
//! The APK contains no Java code. Android's built-in `NativeActivity` loads
//! this library and calls `ANativeActivity_onCreate`, which builds the
//! Custom Tabs intent through JNI, starts it and finishes the activity.
//!
//! The Custom Tab shares Chrome's cookies, so the user's YouTube (Premium)
//! sign-in is used as-is. `ENABLE_URLBAR_HIDING` asks Chrome to slide the
//! toolbar away while scrolling down. An app cannot hide Chrome's toolbar
//! completely or hide the status bar: that is Chrome's window. The share
//! button is turned off (SHARE_STATE_OFF); close, minimize, site-settings and
//! menu buttons and the toolbar height are not configurable by apps.

use jni_sys::{jboolean, jint, jobject, jvalue, JNIEnv, JavaVM, JNI_TRUE};
use std::ffi::{c_void, CStr};
use std::ptr::null_mut;

pub const URL: &CStr = c"https://m.youtube.com/";
const CHROME: &CStr = c"com.android.chrome";
const ACTION_VIEW: &CStr = c"android.intent.action.VIEW";
/// Presence of this extra (even with a null binder) makes the intent a Custom Tab.
const EXTRA_SESSION: &CStr = c"android.support.customtabs.extra.SESSION";
const EXTRA_URLBAR_HIDING: &CStr = c"android.support.customtabs.extra.ENABLE_URLBAR_HIDING";
const EXTRA_TOOLBAR_COLOR: &CStr = c"android.support.customtabs.extra.TOOLBAR_COLOR";
/// SHARE_STATE_OFF (2): no share button in the toolbar (newer Chrome; ignored by older).
const EXTRA_SHARE_STATE: &CStr = c"androidx.browser.customtabs.extra.SHARE_STATE";
const SHARE_STATE_OFF: jint = 2;
/// NO_TITLE (0): no page-title line; the toolbar shows the address only.
const EXTRA_TITLE_VISIBILITY: &CStr = c"android.support.customtabs.extra.TITLE_VISIBILITY";
const NO_TITLE: jint = 0;
/// Dark toolbar matching YouTube's dark theme (#0F0F0F).
pub const TOOLBAR_COLOR: u32 = 0xFF0F_0F0F;

/// Leading fields of `ANativeActivity` (android/native_activity.h); only
/// `env` and `clazz` are read, and the struct is only ever borrowed.
#[repr(C)]
#[allow(dead_code)] // Layout-only fields; read solely in the Android build.
pub struct ANativeActivity {
    callbacks: *mut c_void,
    vm: *mut JavaVM,
    env: *mut JNIEnv,
    clazz: jobject,
}

#[cfg(target_os = "android")]
#[link(name = "android")]
extern "C" {
    fn ANativeActivity_finish(activity: *mut ANativeActivity);
}

/// Entry point called by `android.app.NativeActivity`.
///
/// # Safety
/// Called by the framework on the main thread with a valid activity.
#[cfg(target_os = "android")]
#[no_mangle]
pub unsafe extern "C" fn ANativeActivity_onCreate(
    activity: *mut ANativeActivity,
    _saved_state: *mut c_void,
    _saved_state_size: usize,
) {
    let (env, this) = ((*activity).env, (*activity).clazz);
    // A panic must not unwind into the framework; failing to open is not fatal.
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| open_custom_tab(env, this)));
    ANativeActivity_finish(activity);
}

/// Shorthand for calling a JNIEnv function-table entry.
macro_rules! jni {
    ($env:expr, $f:ident $(, $a:expr)*) => {
        ((**$env).$f.expect(stringify!($f)))($env $(, $a)*)
    };
}

/// Clears a pending Java exception; returns true if there was one.
unsafe fn failed(env: *mut JNIEnv) -> bool {
    if jni!(env, ExceptionCheck) != 0 {
        jni!(env, ExceptionClear);
        true
    } else {
        false
    }
}

/// Opens [`URL`] in a Chrome Custom Tab; falls back to the default browser.
///
/// # Safety
/// `env` must be the calling thread's JNIEnv and `activity` a live Activity.
pub unsafe fn open_custom_tab(env: *mut JNIEnv, activity: jobject) -> Option<()> {
    let s = |text: &CStr| jni!(env, NewStringUTF, text.as_ptr());
    let obj = |o: jobject| jvalue { l: o };

    let uri_class = jni!(env, FindClass, c"android/net/Uri".as_ptr());
    let parse = jni!(
        env,
        GetStaticMethodID,
        uri_class,
        c"parse".as_ptr(),
        c"(Ljava/lang/String;)Landroid/net/Uri;".as_ptr()
    );
    let uri = jni!(
        env,
        CallStaticObjectMethodA,
        uri_class,
        parse,
        [obj(s(URL))].as_ptr()
    );
    if failed(env) || uri.is_null() {
        return None;
    }

    let intent_class = jni!(env, FindClass, c"android/content/Intent".as_ptr());
    let ctor = jni!(
        env,
        GetMethodID,
        intent_class,
        c"<init>".as_ptr(),
        c"(Ljava/lang/String;Landroid/net/Uri;)V".as_ptr()
    );
    let intent = jni!(
        env,
        NewObjectA,
        intent_class,
        ctor,
        [obj(s(ACTION_VIEW)), obj(uri)].as_ptr()
    );
    if failed(env) || intent.is_null() {
        return None;
    }

    // Custom Tab extras.
    let bundle_class = jni!(env, FindClass, c"android/os/Bundle".as_ptr());
    let bundle_ctor = jni!(
        env,
        GetMethodID,
        bundle_class,
        c"<init>".as_ptr(),
        c"()V".as_ptr()
    );
    let bundle = jni!(env, NewObjectA, bundle_class, bundle_ctor, [].as_ptr());
    let put_binder = jni!(
        env,
        GetMethodID,
        bundle_class,
        c"putBinder".as_ptr(),
        c"(Ljava/lang/String;Landroid/os/IBinder;)V".as_ptr()
    );
    jni!(
        env,
        CallVoidMethodA,
        bundle,
        put_binder,
        [obj(s(EXTRA_SESSION)), obj(null_mut())].as_ptr()
    );
    let put_extras = jni!(
        env,
        GetMethodID,
        intent_class,
        c"putExtras".as_ptr(),
        c"(Landroid/os/Bundle;)Landroid/content/Intent;".as_ptr()
    );
    jni!(
        env,
        CallObjectMethodA,
        intent,
        put_extras,
        [obj(bundle)].as_ptr()
    );
    let put_bool = jni!(
        env,
        GetMethodID,
        intent_class,
        c"putExtra".as_ptr(),
        c"(Ljava/lang/String;Z)Landroid/content/Intent;".as_ptr()
    );
    jni!(
        env,
        CallObjectMethodA,
        intent,
        put_bool,
        [
            obj(s(EXTRA_URLBAR_HIDING)),
            jvalue {
                z: JNI_TRUE as jboolean
            }
        ]
        .as_ptr()
    );
    let put_int = jni!(
        env,
        GetMethodID,
        intent_class,
        c"putExtra".as_ptr(),
        c"(Ljava/lang/String;I)Landroid/content/Intent;".as_ptr()
    );
    jni!(
        env,
        CallObjectMethodA,
        intent,
        put_int,
        [
            obj(s(EXTRA_TOOLBAR_COLOR)),
            jvalue {
                i: TOOLBAR_COLOR as jint
            }
        ]
        .as_ptr()
    );
    jni!(
        env,
        CallObjectMethodA,
        intent,
        put_int,
        [obj(s(EXTRA_SHARE_STATE)), jvalue { i: SHARE_STATE_OFF }].as_ptr()
    );
    jni!(
        env,
        CallObjectMethodA,
        intent,
        put_int,
        [obj(s(EXTRA_TITLE_VISIBILITY)), jvalue { i: NO_TITLE }].as_ptr()
    );
    if failed(env) {
        return None;
    }

    let set_package = jni!(
        env,
        GetMethodID,
        intent_class,
        c"setPackage".as_ptr(),
        c"(Ljava/lang/String;)Landroid/content/Intent;".as_ptr()
    );
    let activity_class = jni!(env, GetObjectClass, activity);
    let start = jni!(
        env,
        GetMethodID,
        activity_class,
        c"startActivity".as_ptr(),
        c"(Landroid/content/Intent;)V".as_ptr()
    );
    if failed(env) {
        return None;
    }

    // Prefer Chrome (its cookies hold the YouTube sign-in); if Chrome is
    // missing, let Android pick a browser (ActivityNotFoundException cleared).
    for package in [s(CHROME), null_mut()] {
        jni!(
            env,
            CallObjectMethodA,
            intent,
            set_package,
            [obj(package)].as_ptr()
        );
        jni!(
            env,
            CallVoidMethodA,
            activity,
            start,
            [obj(intent)].as_ptr()
        );
        if !failed(env) {
            return Some(());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constants() {
        assert_eq!(URL.to_str().unwrap(), "https://m.youtube.com/");
        assert_eq!(
            TOOLBAR_COLOR as jint, -15_790_321,
            "ARGB packs into a negative Java int"
        );
        assert!(EXTRA_URLBAR_HIDING
            .to_str()
            .unwrap()
            .ends_with("ENABLE_URLBAR_HIDING"));
    }
}
