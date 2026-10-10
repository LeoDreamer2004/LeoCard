use super::callbacks::IMAGE_PICKER;
use bevy::android::ANDROID_APP;
use jni::objects::{JObject, JValue};
use jni::{JNIEnv, JavaVM};
use std::path::{Path, PathBuf};
use std::sync::mpsc;

pub fn data_directory() -> Option<PathBuf> {
    ANDROID_APP.get()?.internal_data_path()
}

fn with_activity<T>(
    call: impl FnOnce(&mut JNIEnv, &JObject) -> jni::errors::Result<T>,
) -> Result<T, String> {
    let app = ANDROID_APP.get().ok_or("Android Activity 尚未初始化")?;
    // Bevy owns the AndroidApp for the entire native application's lifetime.
    let vm =
        unsafe { JavaVM::from_raw(app.vm_as_ptr().cast()) }.map_err(|error| error.to_string())?;
    let mut env = vm
        .attach_current_thread()
        .map_err(|error| error.to_string())?;
    let activity = unsafe { JObject::from_raw(app.activity_as_ptr().cast()) };
    let result = call(&mut env, &activity);
    if result.is_err() && env.exception_check().unwrap_or(false) {
        let _ = env.exception_describe();
        let _ = env.exception_clear();
    }
    result.map_err(|error| format!("Android 系统接口调用失败：{error}"))
}

pub fn open_url(url: &str) -> Result<(), String> {
    with_activity(|env, activity| {
        let url = env.new_string(url)?;
        env.call_method(
            activity,
            "openUrl",
            "(Ljava/lang/String;)V",
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
            "pickImage",
            "(Ljava/lang/String;)V",
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
            "editText",
            "(JLjava/lang/String;Ljava/lang/String;Z)V",
            &[
                JValue::Long(token as i64),
                JValue::Object(&title),
                JValue::Object(&value),
                JValue::Bool(numeric.into()),
            ],
        )?;
        Ok(())
    })
}

pub fn background_app() -> Result<(), String> {
    with_activity(|env, activity| {
        env.call_method(activity, "backgroundApp", "()V", &[])?;
        Ok(())
    })
}

pub fn install_apk(path: &Path) -> Result<(), String> {
    with_activity(|env, activity| {
        let path = env.new_string(path.to_string_lossy())?;
        env.call_method(
            activity,
            "installApk",
            "(Ljava/lang/String;)V",
            &[JValue::Object(&path)],
        )?;
        Ok(())
    })
}
