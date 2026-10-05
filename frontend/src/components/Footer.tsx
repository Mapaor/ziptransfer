// @ts-nocheck
import Link from "next/link";

import logo from "@/img/icon.png";
import BIcon from "./BIcon";
import Image from 'next/image';
import { tools } from '@/lib/tools';

export default function Footer({ }) {
  return (
    // I do NOT know why the key= trick works I just tried it and it seems to fix the scroll bug LMAOOOOO
    // Edit: didnt work
    <footer className="bg-white --dark:bg-gray-900 z-10 relative" key={"fix-scroll-bug-asdf"}>
      <div className="mx-auto w-full max-w-screen-xl p-4 py-6 lg:py-8">
        <div className="md:flex md:justify-between">
          <div className="mb-6 md:mb-0">
            <a href="#" className="flex items-center">
              <Image src={logo} className="w-8 h-8 me-1" alt="Company Logo" />
              <span className="self-center text-2xl font-semibold whitespace-nowrap --dark:text-white">{process.env.NEXT_PUBLIC_SITE_NAME}</span>
            </a>
          </div>
          <div className="grid grid-cols-2 gap-8 sm:gap-6 sm:grid-cols-3 lg:grid-cols-4">
            <div>
              <h3 className="mb-6 text-sm font-semibold text-gray-900 uppercase --dark:text-white"><Link className="hover:underline" href={"/tools"}>Free Tools</Link></h3>
              <ul className="text-gray-500 --dark:text-gray-400 font-medium">
                {tools.map(t => (
                  <li key={t.slug} className="mb-4">
                    <Link href={`/tools/${t.slug}`} className="hover:underline">
                      {t.title}
                    </Link>
                  </li>
                ))}
              </ul>
            </div>
            <div>
              <p className="mb-6 text-sm font-semibold text-gray-900 uppercase --dark:text-white"><a className="hover:underline" href="/alternative">Alternatives</a></p>
              <ul className="text-gray-500 --dark:text-gray-400 font-medium">
                <li className="mb-4">
                  <h3><Link href="/alternative/wetransfer" className="hover:underline ">WeTransfer Alternative</Link></h3>
                </li>
                <li>
                  <h3><Link href="/alternative/smash" className="hover:underline ">Smash Alternative</Link></h3>
                </li>
              </ul>
            </div>
            <div>
              <p className="mb-6 text-sm font-semibold text-gray-900 uppercase --dark:text-white">Resources</p>
              <ul className="text-gray-500 --dark:text-gray-400 font-medium">
                <li className="mb-4">
                  <a href={`mailto:${process.env.NEXT_PUBLIC_CONTACT_EMAIL}`} className="hover:underline">Contact</a>
                </li>
                <li className="mb-4">
                  <a href={`mailto:${process.env.NEXT_PUBLIC_SUPPORT_EMAIL}`} className="hover:underline">Support</a>
                </li>
                <li className="mb-4">
                  <a href="https://github.com/robinkarlberg/transfer.zip-web?tab=readme-ov-file#self-hosting" className="hover:underline">Self Hosting</a>
                </li>
                <li>
                  <h3><a href="/how-to" className="hover:underline">How-Tos</a></h3>
                </li>
              </ul>
            </div>
            <div>
              <p className="mb-6 text-sm font-semibold text-gray-900 uppercase --dark:text-white"><Link className="hover:underline" href={"/legal"}>Legal</Link></p>
              <ul className="text-gray-500 --dark:text-gray-400 font-medium">
                <li className="mb-4">
                  <Link href="/legal/privacy-policy" className="hover:underline">Privacy Policy</Link>
                </li>
                <li className="mb-4">
                  <Link href="/legal/terms-and-conditions" className="hover:underline">Terms</Link>
                </li>
                <li>
                  <Link href="/legal/impressum" className="hover:underline">Impressum</Link>
                </li>
              </ul>
            </div>
          </div>
        </div>
        <hr className="my-6 border-gray-200 sm:mx-auto --dark:border-gray-700 lg:my-8" />
        <div className="sm:flex sm:items-center sm:justify-between">
          <span className="text-sm text-gray-500 sm:text-center --dark:text-gray-400"><a className="hover:underline" href="#">Shipped with <BIcon name={"heart-fill"} className={"text-red-500"} /></a> from 🇸🇪<BIcon name={"dot"} />&copy; 2025 {process.env.NEXT_PUBLIC_SITE_NAME}.
          </span>
          <div className="flex mt-4 sm:justify-center sm:mt-0">
            
            
            
            
            {process.env.NEXT_PUBLIC_GITHUB_URL && <a href={process.env.NEXT_PUBLIC_GITHUB_URL} className="text-gray-500 hover:text-gray-900 --dark:hover:text-white ms-5">
              <svg className="w-4 h-4" aria-hidden="true" xmlns="http://www.w3.org/2000/svg" fill="currentColor" viewBox="0 0 20 20">
                <path fillRule="evenodd" d="M10 .333A9.911 9.911 0 0 0 6.866 19.65c.5.092.678-.215.678-.477 0-.237-.01-1.017-.014-1.845-2.757.6-3.338-1.169-3.338-1.169a2.627 2.627 0 0 0-1.1-1.451c-.9-.615.07-.6.07-.6a2.084 2.084 0 0 1 1.518 1.021 2.11 2.11 0 0 0 2.884.823c.044-.503.268-.973.63-1.325-2.2-.25-4.516-1.1-4.516-4.9A3.832 3.832 0 0 1 4.7 7.068a3.56 3.56 0 0 1 .095-2.623s.832-.266 2.726 1.016a9.409 9.409 0 0 1 4.962 0c1.89-1.282 2.717-1.016 2.717-1.016.366.83.402 1.768.1 2.623a3.827 3.827 0 0 1 1.02 2.659c0 3.807-2.319 4.644-4.525 4.889a2.366 2.366 0 0 1 .673 1.834c0 1.326-.012 2.394-.012 2.72 0 .263.18.572.681.475A9.911 9.911 0 0 0 10 .333Z" clipRule="evenodd" />
              </svg>
              <span className="sr-only">GitHub account</span>
            </a>}
          </div>
        </div>
      </div>
    </footer>

  )
}

