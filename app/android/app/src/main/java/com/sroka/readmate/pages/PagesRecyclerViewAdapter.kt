package com.sroka.readmate.pages

import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import android.widget.ImageView
import androidx.core.view.isVisible
import androidx.recyclerview.widget.DiffUtil
import androidx.recyclerview.widget.ListAdapter
import androidx.recyclerview.widget.RecyclerView
import com.sroka.readmate.R
import com.sroka.readmate.getFromCacheOrCreate
import uniffi.global_bindings.Page

class PagesRecyclerViewAdapter : ListAdapter<Page, PagesRecyclerViewAdapter.ViewHolder>(DIFF_CALLBACK) {

    companion object {
        private val DIFF_CALLBACK = object : DiffUtil.ItemCallback<Page>() {
            override fun areItemsTheSame(oldItem: Page, newItem: Page): Boolean = oldItem.index() == newItem.index()

            override fun areContentsTheSame(oldItem: Page, newItem: Page) = oldItem.image()?.uid() == newItem.image()?.uid()
        }
    }

    override fun onCreateViewHolder(parent: ViewGroup, viewType: Int): ViewHolder = ViewHolder(
        LayoutInflater.from(parent.context).inflate(R.layout.fragment_page, parent, false)
    )

    override fun onBindViewHolder(holder: ViewHolder, position: Int) {
        val item = getItem(position)
        val image = item.image()
        if (image != null) {
            holder.pageContent.isVisible = true
            holder.pageLoadingError.isVisible = false
            holder.pageContent.setImageBitmap(image.getFromCacheOrCreate())
        } else {
            holder.pageContent.isVisible = false
            holder.pageLoadingError.isVisible = true
            holder.pageContent.setImageBitmap(null)
        }
    }

    inner class ViewHolder(view: View) : RecyclerView.ViewHolder(view) {
        val pageContent: ImageView
        val pageLoadingError: View

        init {
            pageContent = view.findViewById(R.id.page_content)
            pageLoadingError = view.findViewById(R.id.page_loading_error)
        }
    }
}
