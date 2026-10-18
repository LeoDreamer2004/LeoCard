use bevy::android::ANDROID_APP;
use jni::JavaVM;
use jni::objects::JObject;
use rustls_platform_verifier::android::init_with_env;

/// Store the JVM and application class loader before the first HTTPS handshake.
pub fn initialize_https() -> Result<(), String> {
    let app = ANDROID_APP.get().ok_or("Android Activity 尚未初始化")?;
    // AndroidApp owns both pointers for the native application's lifetime.
    let vm = unsafe { JavaVM::from_raw(app.vm_as_ptr().cast()) };
    vm.attach_current_thread(|env| {
        let activity = unsafe { JObject::from_raw(env, app.activity_as_ptr().cast()) };
        init_with_env(env, activity)
    })
    .map_err(|error| format!("无法初始化 Android HTTPS 证书验证器：{error}"))
}
