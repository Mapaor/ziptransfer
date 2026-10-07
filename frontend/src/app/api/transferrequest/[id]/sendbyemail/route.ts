import { NextRequest, NextResponse } from "next/server";
import { sendTransferRequestShare } from "@/lib/server/mail/mail";

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

    // Fetch the transfer request list to find this transfer request
    const workerHost = process.env.NODE_ENV === "development" ? "127.0.0.1" : "api";
    const res = await fetch(`http://${workerHost}:9000/api/transferrequest/list`, {
      headers: {
        ...(cookieHeader ? { cookie: cookieHeader } : {})
      }
    });

    const data = await res.json();
    if (!data.success) {
      return NextResponse.json({ success: false, message: "Could not fetch transfer requests" }, { status: 500 });
    }

    const transferRequest = data.transferRequests.find((tr: any) => tr.id === id);
    if (!transferRequest) {
      return NextResponse.json({ success: false, message: "Transfer request not found" }, { status: 404 });
    }

    const host = request.headers.get("host");
    const protocol = process.env.NODE_ENV === "development" ? "http" : "https";
    const link = `${protocol}://${host}/upload/${transferRequest.secretCode}`;

    // Send the emails
    for (const email of emails) {
      await sendTransferRequestShare(email, {
        name: transferRequest.name || "A transfer request",
        description: transferRequest.description || "",
        link: link,
        brand: null // We don't have brand profiles implemented in the transfer request payload here
      });
    }

    // Note: The frontend types.d.ts doesn't explicitly store emailsSharedWith for TransferRequests,
    // so we don't need to persist the emails array in the database for now.

    return NextResponse.json({ success: true });
  } catch (error: any) {
    console.error("Error sending transfer request email:", error);
    return NextResponse.json({ success: false, message: error.message }, { status: 500 });
  }
}
