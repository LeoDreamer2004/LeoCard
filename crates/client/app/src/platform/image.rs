use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::mpsc;
#[cfg(not(feature = "web"))]
use std::thread;

pub type ImagePickerReceiver = Mutex<mpsc::Receiver<Result<Option<PathBuf>, String>>>;

pub fn request_image(title: &str) -> Result<ImagePickerReceiver, String> {
    #[cfg(feature = "web")]
    return leocard_platform::request_image(title);
    #[cfg(not(feature = "web"))]
    {
        let (sender, receiver) = mpsc::channel();
        let title = title.to_owned();
        thread::Builder::new()
            .name("leocard-image-picker".to_owned())
            .spawn(move || {
                let _ = sender.send(super::pick_image(&title));
            })
            .map_err(|error| format!("无法启动图片选择器：{error}"))?;
        Ok(Mutex::new(receiver))
    }
}
