// @ts-nocheck
import UploadArea from "./UploadArea";
import Header from "@/components/Header";

export default async function ({ params }: { params: Promise<{ secretCode: string }> }) {
  const { secretCode } = await params;
  
  const transferRequest = {
    name: "Upload Request",
    description: "Please upload your files here",
  };

  return (
    <>
      <div className="grid min-h-[100vh] place-items-center ">
        <Header />
        <div className={`bg-white backdrop-blur-sm rounded-2xl border shadow-xl w-full flex flex-col max-w-80`}>
          <div className="p-6">
            <h2 className="font-bold text-xl/8 text-gray-800">{transferRequest.name}</h2>
            <p className="text-gray-600">{transferRequest.description || "No description"}</p>
          </div>
          <hr className="my-2 mx-6" />
          <UploadArea />
        </div>
      </div>
    </>
  )
}