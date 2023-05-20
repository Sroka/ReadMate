package com.sroka.readmate

import android.app.Application
import com.sroka.readmate.books.BooksFragment
import com.sroka.readmate.pages.PagesFragment
import org.koin.android.ext.koin.androidContext
import org.koin.android.ext.koin.androidLogger
import org.koin.core.context.GlobalContext.startKoin
import org.koin.dsl.module
import org.koin.dsl.onClose
import uniffi.global_bindings.BooksStore
import uniffi.global_bindings.GlobalStore
import uniffi.global_bindings.PagesStore

class MainApplication : Application() {
    override fun onCreate() {
        super.onCreate()

        startKoin {
            // Log Koin into Android logger
            androidLogger()
            // Reference Android context
            androidContext(this@MainApplication)
            // Load modules
            modules(globalModule)
        }
    }
}

val globalModule = module {
    single { GlobalStore().apply { init() } }.onClose { it?.destroy() }
    scope<BooksFragment> {
        scoped { BooksStore(globalStore = get()).apply { init() } }.onClose { it?.destroy() }
    }
    scope<PagesFragment> {
        scoped { PagesStore(globalStore = get()).apply { init() } }.onClose { it?.destroy() }
    }
}
