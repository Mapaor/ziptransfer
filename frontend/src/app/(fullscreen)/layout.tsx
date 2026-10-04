import ApplicationProvider from "@/context/ApplicationContext";

export default function FullscreenLayout({ children }: { children: React.ReactNode }) {
  return (
    <div className="min-h-screen">
      <ApplicationProvider>
        {children}
      </ApplicationProvider>
    </div>
  );
}
