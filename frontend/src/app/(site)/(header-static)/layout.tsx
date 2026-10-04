// @ts-nocheck
import Footer from "@/components/Footer";
import Header from "@/components/Header";
import { IS_SELFHOST } from "@/lib/isSelfHosted";

export default function ({ children }: { children: React.ReactNode }) {
  return (
    <div>
      <Header />
      {children}
      {!IS_SELFHOST && <Footer />}
    </div>
  )
}
