package com.sroka.readmate

import android.graphics.Bitmap
import android.util.Log
import java.lang.ref.SoftReference
import java.util.WeakHashMap


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
