// @ts-nocheck
import HashInterceptor from "@/components/HashInterceptor";
import Landing from "./Landing";
import IndieStatement from "@/components/IndieStatement";

export default function () {
  return (
    <div>
      <HashInterceptor />
      <Landing />
      <div className="px-6 sm:px-8">
        <IndieStatement compact />
      </div>
    </div>
  )
}
