// @ts-nocheck
import DownloadArea from "./DownloadArea";
import Header from "@/components/Header";
import { humanFileSize } from "@/lib/transferUtils";

export default async function ({ params }: { params: Promise<{ secretCode: string }> }) {
  const { secretCode } = await params;
  
  const apiUrl = process.env.NODE_ENV === 'development' ? 'http://127.0.0.1:9000/api' : 'http://api:9000/api';
  const res = await fetch(`${apiUrl}/upload/${secretCode}`, { cache: 'no-store' });
  
  if (!res.ok) {
    return (
      <div className="grid min-h-[100vh] place-items-center ">
        <Header />
        <div className="bg-white backdrop-blur-sm rounded-2xl border p-6 shadow-xl w-full max-w-80 min-h-96 flex flex-col justify-center items-center text-center">
          <h2 className="font-bold text-xl/8 text-gray-800">Transfer Not Found</h2>
          <p className="text-gray-600 mt-2">This transfer may have expired or been deleted.</p>
        </div>
      </div>
    )
  }

  const { upload: transfer } = await res.json();
  const totalSize = transfer.files.reduce((acc, f) => acc + f.size, 0);

  return (
    <>
      <div className="grid min-h-[100vh] place-items-center ">
        <Header />
        <div className="bg-white backdrop-blur-sm rounded-2xl border p-6 shadow-xl w-full max-w-80 min-h-96 flex flex-col justify-between">
          <div>
            <h2 className="font-bold text-xl/8 text-gray-800">{transfer.name}</h2>
            <p className="text-gray-600">{transfer.description || "No description"}</p>
            <hr className="my-2" />
            {transfer.files.length > 0 &&
              <span><i className="bi bi-file-earmark me-1"></i>{transfer.files.length} File{transfer.files.length > 1 ? "s" : ""}</span>
            }
            <p className="text-gray-600">{humanFileSize(totalSize, true)}</p>
          </div>
          <div>
            <DownloadArea secretCode={secretCode} />
          </div>
        </div>
      </div>
    </>
  )
}