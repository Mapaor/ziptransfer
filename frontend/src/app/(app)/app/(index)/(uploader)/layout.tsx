export default function ({ children }: { children: React.ReactNode }) {
  return (
    <div className="lg:pl-64 w-full min-h-screen flex flex-col items-stretch sm:bg-gray-50">
      {children}
    </div>
  )
}