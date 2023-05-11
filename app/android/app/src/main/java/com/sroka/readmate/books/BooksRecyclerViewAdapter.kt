package com.sroka.readmate.books

import android.graphics.Bitmap
import android.graphics.Color
import androidx.recyclerview.widget.RecyclerView
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import android.widget.ImageView
import android.widget.ProgressBar
import android.widget.TextView
import androidx.core.view.isGone
import androidx.core.view.isVisible
import androidx.recyclerview.widget.DiffUtil
import androidx.recyclerview.widget.ListAdapter
import com.sroka.readmate.R
import java.nio.ByteBuffer
import java.nio.IntBuffer
import uniffi.global_bindings.BookCover

import uniffi.global_bindings.Pdf


class BooksRecyclerViewAdapter : ListAdapter<Pdf, BooksRecyclerViewAdapter.ViewHolder>(DIFF_CALLBACK) {

    companion object {
        private val DIFF_CALLBACK = object : DiffUtil.ItemCallback<Pdf>() {
            override fun areItemsTheSame(oldItem: Pdf, newItem: Pdf): Boolean {
                val oldItemUuid = when (oldItem) {
                    is Pdf.ErrorPdf -> oldItem.uuid
                    is Pdf.LoadingPdf -> oldItem.uuid
                    is Pdf.ValidPdf -> oldItem.uuid
                }
                val newItemUuid = when (newItem) {
                    is Pdf.ErrorPdf -> newItem.uuid
                    is Pdf.LoadingPdf -> newItem.uuid
                    is Pdf.ValidPdf -> newItem.uuid
                }
                return oldItemUuid == newItemUuid
            }

            override fun areContentsTheSame(oldItem: Pdf, newItem: Pdf): Boolean = oldItem == newItem
        }
    }

    override fun onCreateViewHolder(parent: ViewGroup, viewType: Int): ViewHolder = ViewHolder(
        LayoutInflater.from(parent.context).inflate(R.layout.fragment_books, parent, false)
    )

    override fun onBindViewHolder(holder: ViewHolder, position: Int) {
        val item = getItem(position)
        when (item) {
            is Pdf.ErrorPdf -> {
                holder.bookLoadingProgressBar.isVisible = false
                holder.bookTitle.isVisible = true
                holder.bookTitle.setText(R.string.book_loading_error_title)
            }

            is Pdf.LoadingPdf -> {
                holder.bookLoadingProgressBar.isVisible = true
                holder.bookTitle.isVisible = false
            }

            is Pdf.ValidPdf -> {
                holder.bookLoadingProgressBar.isVisible = false
                holder.bookTitle.isVisible = true
                holder.bookTitle.text = item.title
                when (item.cover) {
                    is BookCover.FirstPage -> {
//                        val sampleBitmap = Bitmap.createBitmap(1, 1, Bitmap.Config.ARGB_8888);
//                        sampleBitmap.eraseColor(Color.RED)
//                        val pixel = sampleBitmap.getPixel(0, 0)
//                        println("PIXEL: ${pixel.toUInt().toString(2)}")
                        val array = item.cover.bitmap.toUIntArray().toIntArray()
                        println("PIXELS: ${array.size}")
                        val createBitmap = Bitmap.createBitmap(array, 100, 141, Bitmap.Config.ARGB_8888);
                        holder.bookCover.setImageBitmap(createBitmap)
                    }

                    BookCover.NoCover -> TODO()
                }
            }
        }

    }

    inner class ViewHolder(view: View) : RecyclerView.ViewHolder(view) {
        val bookCover: ImageView
        val bookTitle: TextView
        val bookLoadingProgressBar: ProgressBar

        init {
            bookCover = view.findViewById(R.id.book_cover)
            bookTitle = view.findViewById(R.id.book_title)
            bookLoadingProgressBar = view.findViewById(R.id.book_loading_progress_bar)
        }
    }
}
