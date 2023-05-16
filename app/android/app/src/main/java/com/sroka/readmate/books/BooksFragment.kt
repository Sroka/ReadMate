package com.sroka.readmate.books

import android.os.Bundle
import android.provider.OpenableColumns
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import android.widget.TextView
import androidx.activity.result.contract.ActivityResultContracts
import androidx.recyclerview.widget.GridLayoutManager
import androidx.recyclerview.widget.RecyclerView
import com.google.android.material.floatingactionbutton.FloatingActionButton
import com.sroka.readmate.IdentityId
import com.sroka.readmate.R
import com.sroka.readmate.assureMainThread
import kotlin.concurrent.thread
import org.koin.android.ext.android.inject
import org.koin.androidx.scope.ScopeFragment
import uniffi.global_bindings.BooksAction
import uniffi.global_bindings.BooksSideEffect
import uniffi.global_bindings.BooksState
import uniffi.global_bindings.BooksStateListener
import uniffi.global_bindings.BooksStore
import uniffi.global_bindings.generatePdfUuid


/**
 * A fragment representing a list of Items.
 */
class BooksFragment : ScopeFragment(), BooksStateListener, IdentityId {

    private val booksStore: BooksStore by inject()

    private var emptyLibraryText: TextView? = null
    private var content: RecyclerView? = null
    private var contentAdapter: BooksRecyclerViewAdapter? = null
    private var addButton: FloatingActionButton? = null

    private val getContent = registerForActivityResult(ActivityResultContracts.GetContent()) { uri ->
        uri?.let { existingUri ->
            thread(true) {
                val pdfUuid = generatePdfUuid()
                view?.post { booksStore.dispatchAction(BooksAction.MarkPdfLoading(uuid = pdfUuid)) }
                val fileName = activity
                    ?.contentResolver
                    ?.query(existingUri, null, null, null, null)
                    ?.use { cursor ->
                        cursor
                            .takeIf { it.moveToFirst() }
                            ?.let { movedCursor ->
                                movedCursor
                                    .getColumnIndex(OpenableColumns.DISPLAY_NAME)
                                    .takeIf { it >= 0 }
                                    ?.let { movedCursor.getString(it) }
                            }

                    } ?: existingUri.lastPathSegment ?: "Unknown file name"

                val bytes = activity
                    ?.contentResolver
                    ?.openInputStream(existingUri)
                    ?.use { it.readBytes() }
                if (bytes != null) {
                    booksStore.dispatchAction(BooksAction.LoadPdf(pdfUuid, fileName, bytes.toUByteArray().asList()))
                } else {
                    booksStore.dispatchAction(BooksAction.MarkPdfLoadingFailed(uuid = pdfUuid))
                }
            }
        }
    }

    override fun onCreateView(
        inflater: LayoutInflater, container: ViewGroup?,
        savedInstanceState: Bundle?,
    ): View? {
        val view = inflater.inflate(R.layout.fragment_books_list, container, false)
        emptyLibraryText = view.findViewById(R.id.empty_library_message)
        content = view.findViewById(R.id.books_list)
        addButton = view.findViewById(R.id.add_book_button)
        val gridLayoutManager = GridLayoutManager(context, 2)
        content?.layoutManager = gridLayoutManager
        contentAdapter = BooksRecyclerViewAdapter()
        content?.adapter = contentAdapter
        addButton?.setOnClickListener { booksStore.dispatchAction(BooksAction.AddClicked) }
        booksStore.addListener(getIdentityId(), this)
        return view
    }

    override fun onDestroyView() {
        booksStore.removeListener(getIdentityId())
        addButton = null
        content = null
        addButton = null
        super.onDestroyView()
    }

    override fun newState(state: BooksState) {
        view?.assureMainThread { render(state) }
    }

    private fun render(state: BooksState) {
        println("New books state: ${Thread.currentThread().name} $state")
        contentAdapter?.submitList(state.books) {
            state.destroy()
        }
    }

    override fun newSideEffect(sideEffect: BooksSideEffect) {
        when (sideEffect) {
            BooksSideEffect.OpenFilePicker -> openFilePicker()
        }
    }

    private fun openFilePicker() = getContent.launch("application/pdf")
}
