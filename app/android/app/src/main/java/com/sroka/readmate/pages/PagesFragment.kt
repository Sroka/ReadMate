package com.sroka.readmate.pages

import android.os.Bundle
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import android.widget.TextView
import androidx.recyclerview.widget.LinearLayoutManager
import androidx.recyclerview.widget.RecyclerView
import androidx.recyclerview.widget.RecyclerView.OnScrollListener
import com.sroka.readmate.IdentityId
import com.sroka.readmate.R
import com.sroka.readmate.assureMainThread
import org.koin.android.ext.android.inject
import org.koin.androidx.scope.ScopeFragment
import uniffi.global_bindings.PagesAction
import uniffi.global_bindings.PagesState
import uniffi.global_bindings.PagesStateListener
import uniffi.global_bindings.PagesStore


/**
 * A fragment representing a list of Items.
 */
class PagesFragment : ScopeFragment(), PagesStateListener, IdentityId {

    companion object {

        private const val BOOK_ID_KEY = "BOOK_ID_KEY"
        fun newInstance(bookId: String): PagesFragment {
            val args = Bundle().apply { putString(BOOK_ID_KEY, bookId) }
            val fragment = PagesFragment()
            fragment.arguments = args
            return fragment
        }

    }

    private val pagesStore: PagesStore by inject()

    private var content: RecyclerView? = null
    private var contentAdapter: PagesRecyclerViewAdapter? = null

    override fun onCreateView(
        inflater: LayoutInflater, container: ViewGroup?,
        savedInstanceState: Bundle?,
    ): View? {
        val view = inflater.inflate(R.layout.fragment_pages_list, container, false)
        content = view.findViewById(R.id.pages_list)
        content?.layoutManager = LinearLayoutManager(context)
        contentAdapter = PagesRecyclerViewAdapter()
        content?.adapter = contentAdapter
        return view
    }

    override fun onViewCreated(view: View, savedInstanceState: Bundle?) {
        super.onViewCreated(view, savedInstanceState)
        pagesStore.addListener(getIdentityId(), this)
        content?.addOnScrollListener(object : OnScrollListener() {
            override fun onScrollStateChanged(recyclerView: RecyclerView, newState: Int) {
                super.onScrollStateChanged(recyclerView, newState)
                (content?.layoutManager as? LinearLayoutManager)?.findLastVisibleItemPosition()?.let {
                    println("onScrollStateChanged - newState: $newState, position: $it")
                    pagesStore.dispatchAction(PagesAction.LoadPage(it))
                }
            }
        })
    }

    override fun onDestroyView() {
        pagesStore.removeListener(getIdentityId())
        super.onDestroyView()
    }

    override fun newState(state: PagesState) {
        view?.assureMainThread { render(state) }
    }

    private fun render(state: PagesState) {
        println("New pages state: ${Thread.currentThread().name} $state")
        contentAdapter?.submitList(state.currentBookPages) {
            state.destroy()
        }
    }
}
