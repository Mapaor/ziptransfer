// @ts-nocheck
"use client"

import { GlobalContext } from "@/context/GlobalContext"
import Link from "next/link"
import { useContext } from "react"

export default function () {
  const { openSignupDialog } = useContext(GlobalContext)


  return (
    <Link
      href="/signin"
      className="rounded-md bg-primary px-3.5 py-2.5 text-sm font-semibold text-white shadow-sm hover:bg-primary-light focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-primary"
    >
      <span>Sign In</span>
      {" "}&rarr;
    </Link>
  )
}

