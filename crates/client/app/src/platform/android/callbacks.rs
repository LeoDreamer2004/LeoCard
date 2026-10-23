use jni::EnvUnowned;
use jni::errors::{LogErrorAndDefault, Result as JniResult};
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

pub fn on_image_result<'local>(
    mut unowned: EnvUnowned<'local>,
    _class: JClass<'local>,
    path: JString<'local>,
    error: JString<'local>,
) {
    unowned
        .with_env(|env| -> JniResult<()> {
            let result = (|| {
                let error = error
                    .try_to_string(env)
                    .map_err(|error| error.to_string())?;
                if !error.is_empty() {
                    return Err(error);
                }
                let path = path.try_to_string(env).map_err(|error| error.to_string())?;
                Ok((!path.is_empty()).then(|| PathBuf::from(path)))
            })();
            if let Some(sender) = IMAGE_PICKER
                .lock()
                .expect("native image result queue poisoned")
                .take()
            {
                let _ = sender.send(result);
            }
            Ok(())
        })
        .resolve::<LogErrorAndDefault>();
}

pub fn on_text_result<'local>(
    mut unowned: EnvUnowned<'local>,
    _class: JClass<'local>,
    token: jlong,
    value: JString<'local>,
    accepted: jboolean,
) {
    unowned
        .with_env(|env| -> JniResult<()> {
            let value = value.try_to_string(env)?;
            TEXT_RESULTS
                .lock()
                .expect("native text result queue poisoned")
                .push_back(NativeTextResult {
                    token: token as u64,
                    value,
                    accepted,
                });
            Ok(())
        })
        .resolve::<LogErrorAndDefault>();
}

static BACK_REQUESTED: AtomicBool = AtomicBool::new(false);

pub fn take_back_request() -> bool {
    BACK_REQUESTED.swap(false, Ordering::Relaxed)
}

pub fn on_back_pressed(_env: EnvUnowned, _class: JClass) {
    BACK_REQUESTED.store(true, Ordering::Relaxed);
}

static AUDIO_ACTIVE: AtomicBool = AtomicBool::new(true);

pub fn audio_active() -> bool {
    AUDIO_ACTIVE.load(Ordering::Relaxed)
}

pub fn on_audio_state(_env: EnvUnowned, _class: JClass, active: jboolean) {
    AUDIO_ACTIVE.store(active, Ordering::Relaxed);
}
