import { NextRequest, NextResponse } from "next/server";
import { sendTransferShare } from "@/lib/server/mail/mail";

export async function POST(
  request: NextRequest,
  { params }: { params: { id: string } }
) {
  try {
    const { id } = params;
    const { emails } = await request.json();

    if (!emails || !Array.isArray(emails) || emails.length === 0) {
      return NextResponse.json({ success: false, message: "Emails are required" }, { status: 400 });
    }

    // Get the user's cookies to authenticate with the backend
    const cookieHeader = request.headers.get('cookie');

    // Fetch the transfer details
    const workerHost = process.env.NODE_ENV === "development" ? "127.0.0.1" : "api";
    const res = await fetch(`http://${workerHost}:9000/api/transfer/${id}`, {
      headers: {
        ...(cookieHeader ? { cookie: cookieHeader } : {})
      }
    });

    const data = await res.json();
    if (!data.success) {
      return NextResponse.json({ success: false, message: "Could not fetch transfer details" }, { status: 500 });
    }

    const transfer = data.transfer;
    if (!transfer) {
      return NextResponse.json({ success: false, message: "Transfer not found" }, { status: 404 });
    }

    const host = request.headers.get("host");
    const protocol = process.env.NODE_ENV === "development" ? "http" : "https";
    const link = `${protocol}://${host}/${transfer.id}`;

    // Send the emails
    for (const email of emails) {
      await sendTransferShare(email, {
        name: transfer.name || "A file transfer",
        description: transfer.description || "",
        link: link,
        brand: null // Assume no brand for now, or fetch brand details if available
      });
    }

    return NextResponse.json({ success: true });
  } catch (error: any) {
    console.error("Error sending transfer email:", error);
    return NextResponse.json({ success: false, message: error.message }, { status: 500 });
  }
}
