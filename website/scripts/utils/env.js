export const localhost = window.location.hostname === "localhost";
const userAgent = navigator.userAgent.toLowerCase();
const iphone = userAgent.includes("iphone");
const ipad = userAgent.includes("ipad");
const touchMac = userAgent.includes("macintosh") && navigator.maxTouchPoints > 1;
export const ios = iphone || ipad || touchMac;
export const canShare = "canShare" in navigator;
