import type { Metadata } from "next";
import { Inter } from "next/font/google";
import { Shell } from "@/components/layout/Shell";
import "./globals.css";

const inter = Inter({ subsets: ["latin"], variable: "--font-inter" });

export const metadata: Metadata = {
  title: "Hindsight — Claude Code Observer",
  description: "20/20 hindsight for your Claude Code sessions",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en" className="dark">
      <body className={`${inter.variable}`}>
        <Shell>{children}</Shell>
      </body>
    </html>
  );
}
