package com.sroka.readmate.books

import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import android.widget.ImageView
import android.widget.ProgressBar
import android.widget.TextView
import androidx.core.view.isVisible
import androidx.recyclerview.widget.DiffUtil
import androidx.recyclerview.widget.ListAdapter
import androidx.recyclerview.widget.RecyclerView
import com.sroka.readmate.R
import com.sroka.readmate.getFromCacheOrCreate
import uniffi.global_bindings.Book
import uniffi.global_bindings.PdfLoadingState


interface BookClickedListener {
    fun onBookClicked(bookId: String)
}

class BooksRecyclerViewAdapter : ListAdapter<Book, BooksRecyclerViewAdapter.ViewHolder>(DIFF_CALLBACK) {

    var listener: BookClickedListener? = null

    companion object {
        private val DIFF_CALLBACK = object : DiffUtil.ItemCallback<Book>() {
            override fun areItemsTheSame(oldItem: Book, newItem: Book): Boolean = oldItem.uuid == newItem.uuid

            override fun areContentsTheSame(oldItem: Book, newItem: Book) = oldItem == newItem
        }
    }

    override fun onCreateViewHolder(parent: ViewGroup, viewType: Int): ViewHolder = ViewHolder(
        LayoutInflater.from(parent.context).inflate(R.layout.fragment_books, parent, false)
    )

    override fun onBindViewHolder(holder: ViewHolder, position: Int) {
        val item = getItem(position)
        holder.bookId = item.uuid
        when (val loadingState = item.loadingState) {
            is PdfLoadingState.ErrorPdf -> {
                holder.bookLoadingError.isVisible = true
                holder.bookLoadingProgressBar.isVisible = false
                holder.bookTitle.isVisible = true
                holder.bookTitle.setText(R.string.book_loading_error_title)
            }

            is PdfLoadingState.LoadingPdf -> {
                holder.bookLoadingError.isVisible = false
                holder.bookLoadingProgressBar.isVisible = true
                holder.bookTitle.isVisible = false
            }

            is PdfLoadingState.ValidPdf -> {
                holder.bookLoadingError.isVisible = false
                holder.bookLoadingProgressBar.isVisible = false
                holder.bookTitle.isVisible = true
                holder.bookTitle.text = loadingState.title
                val thumbnail = item.thumbnail
                if (thumbnail != null) {
                    holder.bookCover.setImageBitmap(thumbnail.getFromCacheOrCreate())
                } else {
                    holder.bookCover.setImageBitmap(null)
                }
            }
        }

    }

    inner class ViewHolder(view: View) : RecyclerView.ViewHolder(view) {
        var bookId: String? = null

        val bookCover: ImageView
        val bookTitle: TextView
        val bookLoadingProgressBar: ProgressBar
        val bookLoadingError: View

        init {
            bookCover = view.findViewById(R.id.book_cover)
            bookTitle = view.findViewById(R.id.book_title)
            bookLoadingProgressBar = view.findViewById(R.id.book_loading_progress_bar)
            bookLoadingError = view.findViewById(R.id.book_loading_error)
            view.setOnClickListener { bookId?.let { listener?.onBookClicked(it) } }
        }
    }
}
