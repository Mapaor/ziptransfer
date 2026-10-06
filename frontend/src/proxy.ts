import { NextResponse } from "next/server"
const selfHostBlacklist = [
  "/api/stripe"
]

const selfHostWhitelist = [
  "/change-password",
  "/app", "/api",
  "/transfer", "/upload",
  "/quick", "/signin", "/signup"
]

const legacyRedirects = [
  { from: "/quick-share", to: "/quick" },
  { from: "/login", to: "/signin" },
  { from: "/about", to: "/" },
]


export function proxy(req) {
  const { pathname } = req.nextUrl



  // Redirect back to /signin if user has no token and wants to use /app
  const token = req.cookies.get("token")
  if (!token && pathname.startsWith("/app")) {
    const newUrl = req.nextUrl.clone()
    newUrl.pathname = "/signin"
    return NextResponse.redirect(newUrl, { status: 302 })
  }

  // legacy redirects
  const legacyMatch = legacyRedirects.find((entry) => {
    return pathname === entry.from
  })
  if (legacyMatch) {
    const newUrl = req.nextUrl.clone()
    newUrl.pathname = legacyMatch.to
    return NextResponse.redirect(newUrl, { status: 301 })
  }

  const newUrl = req.nextUrl.clone()
  // Restrict access to routes when self-hosting
  if (newUrl.pathname === "/") {
    newUrl.pathname = "/quick"
    return NextResponse.redirect(newUrl, { status: 301 })
  }
  if (
    selfHostWhitelist.every((prefix) => !pathname.startsWith(prefix)) ||
    selfHostBlacklist.some((prefix) => pathname.startsWith(prefix))
  ) {
    newUrl.pathname = "/"
    return NextResponse.redirect(newUrl, { status: 301 })
  }

  return NextResponse.next()
}

export const config = {
  matcher: ["/((?!_next|favicon.ico|robots.txt|sitemap.xml|static|img|images|assets|sw\.js|mitm.html).*)"]
}