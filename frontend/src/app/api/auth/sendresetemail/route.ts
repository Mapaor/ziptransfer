import { NextRequest, NextResponse } from "next/server";
import { sendPasswordReset } from "@/lib/server/mail/mail";

export async function POST(request: NextRequest) {
  try {
    const { email, token } = await request.json();

    if (!email || !token) {
      return NextResponse.json({ success: false, message: "Email and token are required" }, { status: 400 });
    }

    const host = request.headers.get("host");
    const protocol = process.env.NODE_ENV === "development" ? "http" : "https";
    // The change-password page expects the URL to be /change-password#<base64_encoded_email_and_token>
    const encodedToken = Buffer.from(`${email} ${token}`).toString("base64");
    const link = `${protocol}://${host}/change-password#${encodedToken}`;

    await sendPasswordReset(email, { link });

    return NextResponse.json({ success: true });
  } catch (error: any) {
    console.error("Error sending password reset email:", error);
    return NextResponse.json({ success: false, message: error.message }, { status: 500 });
  }
}
