use super::callbacks::IMAGE_PICKER;
use bevy::android::ANDROID_APP;
use jni::errors::Result as JniResult;
use jni::objects::{JObject, JValue};
use jni::{Env, JavaVM, jni_sig, jni_str};
use std::path::{Path, PathBuf};
use std::sync::mpsc;

pub fn data_directory() -> Option<PathBuf> {
    ANDROID_APP.get()?.internal_data_path()
}

fn with_activity<T>(call: impl FnOnce(&mut Env, &JObject) -> JniResult<T>) -> Result<T, String> {
    let app = ANDROID_APP.get().ok_or("Android Activity 尚未初始化")?;
    // Bevy owns the AndroidApp for the entire native application's lifetime.
    let vm = unsafe { JavaVM::from_raw(app.vm_as_ptr().cast()) };
    vm.attach_current_thread(|env| {
        let activity = unsafe { JObject::from_raw(env, app.activity_as_ptr().cast()) };
        call(env, &activity)
    })
    .map_err(|error| format!("Android 系统接口调用失败：{error}"))
}

pub fn open_url(url: &str) -> Result<(), String> {
    with_activity(|env, activity| {
        let url = env.new_string(url)?;
        env.call_method(
            activity,
            jni_str!("openUrl"),
            jni_sig!((java.lang.String) -> void),
            &[JValue::Object(&url)],
        )?;
        Ok(())
    })
}

/// Called on the existing picker worker, never on Bevy's render or Android's UI thread.
pub fn pick_image(title: &str) -> Result<Option<PathBuf>, String> {
    let (sender, receiver) = mpsc::channel();
    {
        let mut pending = IMAGE_PICKER.lock().map_err(|_| "图片选择器锁定失败")?;
        if pending.is_some() {
            return Err("已有图片选择器打开".to_owned());
        }
        *pending = Some(sender);
    }
    let launched = with_activity(|env, activity| {
        let title = env.new_string(title)?;
        env.call_method(
            activity,
            jni_str!("pickImage"),
            jni_sig!((java.lang.String) -> void),
            &[JValue::Object(&title)],
        )?;
        Ok(())
    });
    if let Err(error) = launched {
        *IMAGE_PICKER.lock().map_err(|_| "图片选择器锁定失败")? = None;
        return Err(error);
    }
    receiver.recv().map_err(|_| "图片选择器已关闭".to_owned())?
}

pub fn show_text_editor(token: u64, title: &str, value: &str, numeric: bool) -> Result<(), String> {
    with_activity(|env, activity| {
        let title = env.new_string(title)?;
        let value = env.new_string(value)?;
        env.call_method(
            activity,
            jni_str!("editText"),
            jni_sig!((long, java.lang.String, java.lang.String, boolean) -> void),
            &[
                JValue::Long(token as i64),
                JValue::Object(&title),
                JValue::Object(&value),
                JValue::Bool(numeric),
            ],
        )?;
        Ok(())
    })
}

pub fn background_app() -> Result<(), String> {
    with_activity(|env, activity| {
        env.call_method(
            activity,
            jni_str!("backgroundApp"),
            jni_sig!(() -> void),
            &[],
        )?;
        Ok(())
    })
}

pub fn install_apk(path: &Path) -> Result<(), String> {
    with_activity(|env, activity| {
        let path = env.new_string(path.to_string_lossy())?;
        env.call_method(
            activity,
            jni_str!("installApk"),
            jni_sig!((java.lang.String) -> void),
            &[JValue::Object(&path)],
        )?;
        Ok(())
    })
}
