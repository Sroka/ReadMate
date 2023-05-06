package com.sroka.readmate

interface IdentityId {
    fun getIdentityId(): String = System.identityHashCode(this).toString()
}
