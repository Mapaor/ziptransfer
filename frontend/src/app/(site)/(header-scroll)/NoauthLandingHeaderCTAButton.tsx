// @ts-nocheck
"use client"

import { GlobalContext } from "@/context/GlobalContext"
import Link from "next/link"
import { useContext } from "react"

export default function () {
  const { openSignupDialog } = useContext(GlobalContext)


  return (
    <Link href={"/signin"} className="text-sm/6 font-semibold text-white rounded-full bg-primary px-4 py-2 hover:bg-primary-light">
      Sign In <span aria-hidden="true">&rarr;</span>
    </Link>
  )
}

