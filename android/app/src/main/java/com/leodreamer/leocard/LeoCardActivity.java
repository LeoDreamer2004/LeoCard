package com.leodreamer.leocard;

import android.app.AlertDialog;
import android.content.Intent;
import android.net.Uri;
import android.media.AudioManager;
import android.media.AudioAttributes;
import android.media.AudioFocusRequest;
import android.os.Bundle;
import android.text.InputType;
import android.view.KeyEvent;
import android.view.WindowInsets;
import android.view.WindowInsetsController;
import android.view.WindowManager;
import android.widget.EditText;
import com.google.androidgamesdk.GameActivity;
import androidx.activity.OnBackPressedCallback;
import androidx.activity.result.ActivityResultLauncher;
import androidx.activity.result.contract.ActivityResultContracts;
import androidx.core.content.FileProvider;
import java.io.File;
import java.io.FileOutputStream;
import java.io.InputStream;

/** Android owns text editing, URI permissions and document access. Rust owns game state. */
public final class LeoCardActivity extends GameActivity {
    static { System.loadLibrary("leocard_client"); }
    private static final int MAX_IMAGE_BYTES = 16 * 1024 * 1024;
    private boolean pickingImage;
    private ActivityResultLauncher<Intent> imagePicker;
    private AlertDialog editorDialog;
    private AudioManager audioManager;
    private AudioFocusRequest audioFocus;
    private boolean resumed;

    private static native void audioState(boolean active);
    private static native void backPressed();
    private static native void imageResult(String path, String error);
    private static native void textResult(long token, String value, boolean accepted);

    @Override public void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        getWindow().addFlags(WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON);
        audioManager = getSystemService(AudioManager.class);
        audioFocus = new AudioFocusRequest.Builder(AudioManager.AUDIOFOCUS_GAIN)
            .setAudioAttributes(new AudioAttributes.Builder().setUsage(AudioAttributes.USAGE_GAME)
                .setContentType(AudioAttributes.CONTENT_TYPE_SONIFICATION).build())
            .setOnAudioFocusChangeListener(change -> audioState(resumed && change == AudioManager.AUDIOFOCUS_GAIN))
            .build();
        imagePicker = registerForActivityResult(new ActivityResultContracts.StartActivityForResult(), result -> {
            pickingImage = false;
            Intent data = result.getData();
            if (result.getResultCode() != RESULT_OK || data == null || data.getData() == null) {
                imageResult("", "");
                return;
            }
            Uri uri = data.getData();
            new Thread(() -> importImage(uri), "leocard-image-import").start();
        });
        getOnBackPressedDispatcher().addCallback(this, new OnBackPressedCallback(true) {
            @Override public void handleOnBackPressed() { backPressed(); }
        });
    }

    @Override protected void onResume() {
        super.onResume();
        resumed = true;
        audioState(audioManager.requestAudioFocus(audioFocus) == AudioManager.AUDIOFOCUS_REQUEST_GRANTED);
    }

    @Override public void onWindowFocusChanged(boolean focused) {
        super.onWindowFocusChanged(focused);
        if (!focused) return;
        WindowInsetsController controller = getWindow().getInsetsController();
        if (controller != null) {
            controller.setSystemBarsBehavior(WindowInsetsController.BEHAVIOR_SHOW_TRANSIENT_BARS_BY_SWIPE);
            controller.hide(WindowInsets.Type.systemBars());
        }
    }

    @Override protected void onPause() {
        resumed = false;
        audioState(false);
        audioManager.abandonAudioFocusRequest(audioFocus);
        super.onPause();
    }

    public void backgroundApp() {
        runOnUiThread(() -> moveTaskToBack(true));
    }

    @Override public boolean dispatchKeyEvent(KeyEvent event) {
        // GameActivity forwards keys to native code before AppCompat's back dispatcher.
        if (event.getKeyCode() == KeyEvent.KEYCODE_BACK) {
            if (event.getAction() == KeyEvent.ACTION_UP) backPressed();
            return true;
        }
        return super.dispatchKeyEvent(event);
    }

    public void openUrl(String url) {
        runOnUiThread(() -> {
            try { startActivity(new Intent(Intent.ACTION_VIEW, Uri.parse(url))); }
            catch (Exception error) { android.util.Log.e("LeoCard", "Cannot open browser", error); }
        });
    }

    public void installApk(String path) {
        runOnUiThread(() -> {
            Uri uri = FileProvider.getUriForFile(this, getPackageName() + ".files", new File(path));
            Intent install = new Intent(Intent.ACTION_VIEW);
            install.setDataAndType(uri, "application/vnd.android.package-archive");
            install.addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION);
            startActivity(install);
        });
    }

    public void pickImage(String title) {
        runOnUiThread(() -> {
            pickingImage = true;
            Intent intent = new Intent(Intent.ACTION_OPEN_DOCUMENT);
            intent.setType("image/*");
            intent.addCategory(Intent.CATEGORY_OPENABLE);
            intent.putExtra(Intent.EXTRA_MIME_TYPES, new String[]{"image/png", "image/jpeg"});
            try { imagePicker.launch(Intent.createChooser(intent, title)); }
            catch (Exception error) {
                pickingImage = false;
                imageResult("", "无法打开图片选择器：" + error.getMessage());
            }
        });
    }

    private void importImage(Uri uri) {
        File target = null;
        try {
            File directory = new File(getFilesDir(), "images");
            if (!directory.isDirectory() && !directory.mkdirs()) throw new Exception("无法创建图片目录");
            target = File.createTempFile("selected-", ".image", directory);
            try (InputStream input = getContentResolver().openInputStream(uri);
                 FileOutputStream output = new FileOutputStream(target)) {
                if (input == null) throw new Exception("无法读取所选图片");
                byte[] buffer = new byte[8192];
                int total = 0, count;
                while ((count = input.read(buffer)) != -1) {
                    total += count;
                    if (total > MAX_IMAGE_BYTES) throw new Exception("图片不能超过 16 MB");
                    output.write(buffer, 0, count);
                }
            }
            imageResult(target.getAbsolutePath(), "");
        } catch (Exception error) {
            if (target != null) target.delete();
            imageResult("", "无法导入图片：" + error.getMessage());
        }
    }

    public void editText(long token, String title, String value, boolean numeric) {
        runOnUiThread(() -> {
            if (editorDialog != null) editorDialog.cancel();
            EditText editor = new EditText(this);
            editor.setSingleLine(true);
            editor.setInputType(numeric ? InputType.TYPE_CLASS_NUMBER :
                InputType.TYPE_CLASS_TEXT | InputType.TYPE_TEXT_FLAG_CAP_SENTENCES);
            editor.setText(value);
            editor.setSelection(editor.length());
            int padding = (int)(24 * getResources().getDisplayMetrics().density);
            editor.setPadding(padding, padding / 2, padding, padding / 2);
            editorDialog = new AlertDialog.Builder(this).setTitle(title).setView(editor)
                .setPositiveButton("确定", (dialog, which) -> textResult(token, editor.getText().toString(), true))
                .setNegativeButton("取消", (dialog, which) -> textResult(token, value, false))
                .setOnCancelListener(dialog -> textResult(token, value, false)).create();
            editorDialog.setOnDismissListener(dialog -> editorDialog = null);
            editorDialog.getWindow().setSoftInputMode(WindowManager.LayoutParams.SOFT_INPUT_STATE_ALWAYS_VISIBLE);
            editorDialog.show();
            editor.requestFocus();
        });
    }

    @Override protected void onDestroy() {
        if (pickingImage) imageResult("", "");
        if (editorDialog != null) editorDialog.cancel();
        super.onDestroy();
    }
}
