// @ts-nocheck
import TransfersPage from "./TransfersPage"
import TransferSidebarWrapper from "./TransferSidebarWrapper"

import { cookies } from "next/headers"

export default async function Page({ params }: { params: Promise<{ transferIdSlug?: string[] }> }) {
  try {
    console.log("page.js running");
    const cookieStore = await cookies();
    const token = cookieStore.get("token")?.value;

    let transfers = [];
    if (token) {
      try {
        const backendUrl = process.env.NODE_ENV === 'development' 
          ? 'http://127.0.0.1:9000' 
          : 'http://api:9000';
          
        const res = await fetch(`${backendUrl}/api/transfer/list`, {
          headers: {
            Cookie: `token=${token}`
          },
          cache: 'no-store'
        });
        const data = await res.json();
        if (data.success) {
          transfers = data.transfers;
        }
      } catch (e) {
        console.error("Failed to fetch transfers", e);
      }
    }

    const transferRequestsWithCount = []
    const resolvedParams = await params;
    const selectedTransferId = resolvedParams?.transferIdSlug?.[0];
    const selectedTransfer = selectedTransferId ? transfers.find(t => t.id === selectedTransferId) : null;
    
    console.log("page.js rendering");

    return (
      <>
        <TransfersPage
          transfers={transfers}
          transferRequests={transferRequestsWithCount}
        />
        <TransferSidebarWrapper
          user={{}}
          transfer={selectedTransfer}
        />
      </>
    )
  } catch (error) {
    console.error("page.js crashed with:", error);
    return <div>Error loading page.</div>;
  }
}