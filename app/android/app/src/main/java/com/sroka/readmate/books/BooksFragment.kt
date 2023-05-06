package com.sroka.readmate.books

import android.os.Bundle
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
import com.sroka.readmate.books.placeholder.PlaceholderContent
import org.koin.android.ext.android.inject
import org.koin.androidx.scope.ScopeFragment
import uniffi.global_bindings.BooksSideEffect
import uniffi.global_bindings.BooksState
import uniffi.global_bindings.BooksStateListener
import uniffi.global_bindings.BooksStore
import uniffi.global_bindings.BooksThunk
import uniffi.global_bindings.GlobalStore
import uniffi.global_bindings.GlobalThunk


/**
 * A fragment representing a list of Items.
 */
class BooksFragment : ScopeFragment(), BooksStateListener, IdentityId {

    private val booksStore: BooksStore by inject()
    private val globalStore: GlobalStore by inject()

    private var emptyLibraryText: TextView? = null
    private var content: RecyclerView? = null
    private var addButton: FloatingActionButton? = null

    private val getContent = registerForActivityResult(ActivityResultContracts.GetContent()) { uri ->
        uri?.let { existingUri ->
            requireActivity()
                .contentResolver
                .openInputStream(existingUri)
                ?.use { it.readBytes() }
                ?.let { globalStore.dispatchThunk(GlobalThunk.LoadPdf(it.toUByteArray().asList())) }
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
        content?.layoutManager = GridLayoutManager(context, 2)
        content?.adapter = BooksRecyclerViewAdapter(PlaceholderContent.ITEMS)
        addButton?.setOnClickListener { booksStore.dispatchThunk(BooksThunk.AddClicked) }
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
        println("New books state: $state")
    }

    override fun newSideEffect(sideEffect: BooksSideEffect) {
        when (sideEffect) {
            BooksSideEffect.OpenFilePicker -> openFilePicker()
        }
    }

    private fun openFilePicker() = getContent.launch("application/pdf")
}
