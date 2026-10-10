use jni::JNIEnv;
use jni::objects::{JClass, JString};
use jni::sys::{jboolean, jlong};
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, mpsc::Sender};

type ImageResult = Result<Option<PathBuf>, String>;
pub(super) static IMAGE_PICKER: Mutex<Option<Sender<ImageResult>>> = Mutex::new(None);
static TEXT_RESULTS: Mutex<VecDeque<NativeTextResult>> = Mutex::new(VecDeque::new());

pub struct NativeTextResult {
    pub token: u64,
    pub value: String,
    pub accepted: bool,
}

pub fn take_text_results() -> Vec<NativeTextResult> {
    TEXT_RESULTS
        .lock()
        .expect("native text result queue poisoned")
        .drain(..)
        .collect()
}

#[unsafe(no_mangle)]
extern "system" fn Java_com_leodreamer_leocard_LeoCardActivity_imageResult(
    mut env: JNIEnv,
    _class: JClass,
    path: JString,
    error: JString,
) {
    let result = (|| {
        let error: String = env.get_string(&error).map_err(|e| e.to_string())?.into();
        if !error.is_empty() {
            return Err(error);
        }
        let path: String = env.get_string(&path).map_err(|e| e.to_string())?.into();
        Ok((!path.is_empty()).then(|| PathBuf::from(path)))
    })();
    if let Some(sender) = IMAGE_PICKER
        .lock()
        .expect("native image result queue poisoned")
        .take()
    {
        let _ = sender.send(result);
    }
}

#[unsafe(no_mangle)]
extern "system" fn Java_com_leodreamer_leocard_LeoCardActivity_textResult(
    mut env: JNIEnv,
    _class: JClass,
    token: jlong,
    value: JString,
    accepted: jboolean,
) {
    if let Ok(value) = env.get_string(&value) {
        TEXT_RESULTS
            .lock()
            .expect("native text result queue poisoned")
            .push_back(NativeTextResult {
                token: token as u64,
                value: value.into(),
                accepted: accepted != 0,
            });
    }
}

static BACK_REQUESTED: AtomicBool = AtomicBool::new(false);

pub fn take_back_request() -> bool {
    BACK_REQUESTED.swap(false, Ordering::Relaxed)
}

#[unsafe(no_mangle)]
extern "system" fn Java_com_leodreamer_leocard_LeoCardActivity_backPressed(
    _env: JNIEnv,
    _class: JClass,
) {
    BACK_REQUESTED.store(true, Ordering::Relaxed);
}

static AUDIO_ACTIVE: AtomicBool = AtomicBool::new(true);

pub fn audio_active() -> bool {
    AUDIO_ACTIVE.load(Ordering::Relaxed)
}

#[unsafe(no_mangle)]
extern "system" fn Java_com_leodreamer_leocard_LeoCardActivity_audioState(
    _env: JNIEnv,
    _class: JClass,
    active: jboolean,
) {
    AUDIO_ACTIVE.store(active != 0, Ordering::Relaxed);
}
