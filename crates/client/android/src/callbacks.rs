use jni::EnvUnowned;
use jni::objects::{JClass, JString};
use jni::sys::{jboolean, jlong};
use leocard_client::platform;

#[unsafe(no_mangle)]
extern "system" fn Java_com_leodreamer_leocard_LeoCardActivity_imageResult<'local>(
    env: EnvUnowned<'local>,
    class: JClass<'local>,
    path: JString<'local>,
    error: JString<'local>,
) {
    platform::on_image_result(env, class, path, error);
}

#[unsafe(no_mangle)]
extern "system" fn Java_com_leodreamer_leocard_LeoCardActivity_textResult<'local>(
    env: EnvUnowned<'local>,
    class: JClass<'local>,
    token: jlong,
    value: JString<'local>,
    accepted: jboolean,
) {
    platform::on_text_result(env, class, token, value, accepted);
}

#[unsafe(no_mangle)]
extern "system" fn Java_com_leodreamer_leocard_LeoCardActivity_backPressed(
    env: EnvUnowned,
    class: JClass,
) {
    platform::on_back_pressed(env, class);
}

#[unsafe(no_mangle)]
extern "system" fn Java_com_leodreamer_leocard_LeoCardActivity_audioState(
    env: EnvUnowned,
    class: JClass,
    active: jboolean,
) {
    platform::on_audio_state(env, class, active);
}
