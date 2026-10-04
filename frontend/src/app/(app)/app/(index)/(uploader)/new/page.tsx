// @ts-nocheck
import { useServerAuth } from "@/lib/server/wrappers/auth";
import NewTransferFileUpload from "../../../../../../components/dashboard/NewTransferFileUpload";

import "./bg.css"

export default async function () {
  const { user } = await useServerAuth()
  const storage = await user.getStorage()
  const brandProfiles = []
  return (
    <div className="min-h-screen flex flex-col items-stretch bg-white new-transfer-background">
      <div className="">
        <div className="mx-auto max-w-xl mb-4 sm:mb-16 px-4">
          <h1 className={`text-center text-4xl sm:text-6xl font-bold mt-8 sm:mt-12 tracking-tight text-gray-800`}>
            New Transfer
          </h1>
          <p className="text-center text-gray-500 mt-4 text-base sm:text-xl">
            Easily send your files, or request others to send files to you. 
          </p>
        </div>
        <NewTransferFileUpload user={user.friendlyObj()} storage={storage} brandProfiles={brandProfiles} />
      </div>
    </div>
  )
}
