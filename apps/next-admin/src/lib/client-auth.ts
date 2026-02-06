export function getCookieValue(name: string) {
  const pair = document.cookie
    .split("; ")
    .find((row) => row.startsWith(`${name}=`));
  return pair ? decodeURIComponent(pair.split("=")[1]) : undefined;
}

export function getClientAuth() {
  return {
    token: getCookieValue("rustok-admin-token"),
    tenant: getCookieValue("rustok-admin-tenant") ?? "demo",
  };
}
