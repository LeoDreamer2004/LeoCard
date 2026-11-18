package com.leodreamer.leocard;

import android.database.Cursor;
import android.database.MatrixCursor;
import android.os.CancellationSignal;
import android.os.ParcelFileDescriptor;
import android.provider.DocumentsContract.Document;
import android.provider.DocumentsContract.Root;
import android.provider.DocumentsProvider;
import java.io.File;
import java.io.FileNotFoundException;
import java.io.IOException;

/** Exposes only editable word libraries, never profiles, identities or other private settings. */
public class WordLibraryDocumentsProvider extends DocumentsProvider {
    private static final String ROOT_ID = "libraries";
    private static final String[] ROOT_COLUMNS = { Root.COLUMN_ROOT_ID, Root.COLUMN_DOCUMENT_ID,
        Root.COLUMN_TITLE, Root.COLUMN_FLAGS, Root.COLUMN_MIME_TYPES };
    private static final String[] DOCUMENT_COLUMNS = { Document.COLUMN_DOCUMENT_ID,
        Document.COLUMN_DISPLAY_NAME, Document.COLUMN_MIME_TYPE, Document.COLUMN_FLAGS,
        Document.COLUMN_SIZE, Document.COLUMN_LAST_MODIFIED };

    @Override public boolean onCreate() { return true; }

    private File directory() {
        File directory = new File(getContext().getFilesDir(), "draw-guess/word-libraries");
        directory.mkdirs();
        return directory;
    }

    private File resolve(String id) throws FileNotFoundException {
        if (ROOT_ID.equals(id)) return directory();
        if (!id.startsWith("library:")) throw new FileNotFoundException("Unknown document");
        String name = id.substring("library:".length());
        if (!name.endsWith(".json") || name.contains("/") || name.contains("\\"))
            throw new FileNotFoundException("Invalid library filename");
        File file = new File(directory(), name);
        try {
            if (!file.getCanonicalFile().getParentFile().equals(directory().getCanonicalFile()))
                throw new FileNotFoundException("Outside word library directory");
        } catch (IOException error) { throw new FileNotFoundException(error.getMessage()); }
        return file;
    }

    private void row(MatrixCursor cursor, Object... fields) {
        Object[] values = new Object[cursor.getColumnCount()];
        for (int i = 0; i < fields.length; i += 2) {
            int column = cursor.getColumnIndex((String)fields[i]);
            if (column >= 0) values[column] = fields[i + 1];
        }
        cursor.addRow(values);
    }

    @Override public Cursor queryRoots(String[] projection) {
        MatrixCursor cursor = new MatrixCursor(projection == null ? ROOT_COLUMNS : projection);
        row(cursor, Root.COLUMN_ROOT_ID, ROOT_ID, Root.COLUMN_DOCUMENT_ID, ROOT_ID,
            Root.COLUMN_TITLE, "LeoCard 词库", Root.COLUMN_FLAGS,
            Root.FLAG_LOCAL_ONLY | Root.FLAG_SUPPORTS_CREATE, Root.COLUMN_MIME_TYPES, "application/json");
        return cursor;
    }

    private void include(MatrixCursor cursor, String id) throws FileNotFoundException {
        File file = resolve(id);
        boolean root = ROOT_ID.equals(id);
        row(cursor, Document.COLUMN_DOCUMENT_ID, id,
            Document.COLUMN_DISPLAY_NAME, root ? "LeoCard 词库" : file.getName(),
            Document.COLUMN_MIME_TYPE, root ? Document.MIME_TYPE_DIR : "application/json",
            Document.COLUMN_FLAGS, root ? Document.FLAG_DIR_SUPPORTS_CREATE :
                Document.FLAG_SUPPORTS_WRITE | Document.FLAG_SUPPORTS_DELETE,
            Document.COLUMN_SIZE, root ? 0L : file.length(), Document.COLUMN_LAST_MODIFIED, file.lastModified());
    }

    @Override public Cursor queryDocument(String id, String[] projection) throws FileNotFoundException {
        MatrixCursor cursor = new MatrixCursor(projection == null ? DOCUMENT_COLUMNS : projection);
        include(cursor, id);
        return cursor;
    }

    @Override public Cursor queryChildDocuments(String parent, String[] projection, String sortOrder)
            throws FileNotFoundException {
        if (!ROOT_ID.equals(parent)) throw new FileNotFoundException("Not a directory");
        MatrixCursor cursor = new MatrixCursor(projection == null ? DOCUMENT_COLUMNS : projection);
        File[] children = directory().listFiles();
        if (children != null) for (File file : children) {
            if (!file.isFile() || !file.getName().endsWith(".json")) continue;
            try { include(cursor, "library:" + file.getName()); }
            catch (FileNotFoundException invalid) { /* Ignore links outside this directory. */ }
        }
        return cursor;
    }

    @Override public ParcelFileDescriptor openDocument(String id, String mode, CancellationSignal signal)
            throws FileNotFoundException {
        if (ROOT_ID.equals(id)) throw new FileNotFoundException("Cannot open a directory as a file");
        return ParcelFileDescriptor.open(resolve(id), ParcelFileDescriptor.parseMode(mode));
    }

    @Override public String createDocument(String parent, String mimeType, String name)
            throws FileNotFoundException {
        if (!ROOT_ID.equals(parent)) throw new FileNotFoundException("Invalid parent");
        if (!name.endsWith(".json")) name += ".json";
        File file = resolve("library:" + name);
        try {
            if (!file.createNewFile()) throw new FileNotFoundException("File already exists");
        } catch (IOException error) { throw new FileNotFoundException(error.getMessage()); }
        return "library:" + name;
    }

    @Override public void deleteDocument(String id) throws FileNotFoundException {
        if (ROOT_ID.equals(id) || !resolve(id).delete()) throw new FileNotFoundException("Cannot delete document");
    }

    @Override public boolean isChildDocument(String parent, String child) {
        if (!ROOT_ID.equals(parent) || ROOT_ID.equals(child)) return false;
        try { return resolve(child).isFile(); }
        catch (FileNotFoundException error) { return false; }
    }
}
