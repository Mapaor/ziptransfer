import { cookies } from 'next/headers';

export async function useServerAuth() {
  const cookieStore = await cookies();
  const token = cookieStore.get('token')?.value;
  return {
    token: "mock-token",
    id: "guest",
    user: {
      _id: "guest",
      name: "Guest",
      email: "guest@example.com",
      planStatus: "active",
      plan: "pro",
      getPlan: () => "pro",
      getStorage: async () => ({ usedBytes: 0, totalBytes: 500 * 1024 * 1024 * 1024 }),
      friendlyObj: function() { return { _id: this._id, name: this.name, email: this.email, planStatus: this.planStatus, plan: this.plan } }
    },
    isGuestUser: false,
    isFreeUser: false,
    session: null
  };
}
