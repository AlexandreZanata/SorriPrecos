package br.com.sorriprecos

import android.content.ContentValues
import android.content.Context
import android.database.sqlite.SQLiteDatabase
import android.database.sqlite.SQLiteOpenHelper

internal class AndroidCatalogueCache(context: Context) : SQLiteOpenHelper(
    context.applicationContext, "sorriprecos-cache.db", null, 1
), LocalStore {
    override fun onCreate(db: SQLiteDatabase) {
        db.execSQL("CREATE TABLE cache (key TEXT PRIMARY KEY NOT NULL, value TEXT NOT NULL)")
    }

    override fun onUpgrade(db: SQLiteDatabase, oldVersion: Int, newVersion: Int) = Unit

    override fun get(key: String): String? =
        readableDatabase.rawQuery("SELECT value FROM cache WHERE key = ?", arrayOf(key)).use {
            if (it.moveToFirst()) it.getString(0) else null
        }

    override fun put(key: String, value: String) {
        val row = ContentValues().apply {
            put("key", key)
            put("value", value)
        }
        writableDatabase.replaceOrThrow("cache", null, row)
    }

    override fun remove(key: String) {
        writableDatabase.delete("cache", "key = ?", arrayOf(key))
    }

    override fun keys(prefix: String): List<String> =
        readableDatabase.rawQuery("SELECT key FROM cache WHERE key >= ? AND key < ?", arrayOf(prefix, prefix.dropLast(1) + ";")).use {
            buildList { while (it.moveToNext()) add(it.getString(0)) }
        }
}
