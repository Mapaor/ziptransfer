"use client"

import { createContext, useState } from "react"

export const GlobalContext = createContext({})

export default function GlobalProvider({ children }: { children: React.ReactNode }) {

    const openSignupDialog = (files) => {
        window.location.href = "/signup"
    }

    return (
        <GlobalContext.Provider value={{
            openSignupDialog
        }}>
            {children}
        </GlobalContext.Provider >
    );
};