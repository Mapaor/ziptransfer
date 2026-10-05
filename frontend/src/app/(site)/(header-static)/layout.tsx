// @ts-nocheck
import Footer from "@/components/Footer";
import Header from "@/components/Header";

export default function ({ children }: { children: React.ReactNode }) {
  return (
    <div>
      <Header />
      {children}
      <Footer />
    </div>
  )
}

