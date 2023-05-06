package com.sroka.readmate

import android.os.Bundle
import androidx.appcompat.app.AppCompatActivity
import com.sroka.readmate.books.BooksFragment
import com.sroka.readmate.R
import org.koin.android.ext.android.inject
import uniffi.global_bindings.GlobalState
import uniffi.global_bindings.GlobalStateListener
import uniffi.global_bindings.GlobalStore

class MainActivity : AppCompatActivity(), GlobalStateListener, IdentityId {

    private val globalStore: GlobalStore by inject()

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_main)
        globalStore.addListener(getIdentityId(), this)
        supportFragmentManager
            .beginTransaction()
            .replace(R.id.fragment_container, BooksFragment())
            .commit()
    }

    override fun onDestroy() {
        globalStore.removeListener(getIdentityId())
        super.onDestroy()
    }

    override fun newState(state: GlobalState) {
        println("New global state: $state")
    }
}
