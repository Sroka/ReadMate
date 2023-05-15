package com.sroka.readmate

import android.graphics.Bitmap
import android.os.Looper
import android.util.Log
import android.view.View
import java.lang.ref.SoftReference


private val bitmapMap = mutableMapOf<String, SoftReference<Bitmap>>()
fun uniffi.global_bindings.Bitmap.getFromCacheOrCreate(): Bitmap {
    val uid = uid()
    val existingBitmap = bitmapMap[uid]?.get()
    return if (existingBitmap != null) {
        existingBitmap
    } else {
        Log.w("UNIFFI_BITMAP_UTIL", "Bitmap ${uid} not cached - creating a new one")
        val pixels = copyPixels().toUIntArray().toIntArray()
        val newBitmap = Bitmap.createBitmap(pixels, width(), height(), Bitmap.Config.ARGB_8888);
        bitmapMap[uid] = SoftReference(newBitmap)
        newBitmap
    }
}

fun View.assureMainThread(block: () -> Unit) {
    if (Thread.currentThread() == Looper.getMainLooper().thread) block() else post { block() }
}
