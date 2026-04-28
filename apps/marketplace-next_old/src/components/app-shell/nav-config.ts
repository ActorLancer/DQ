import { Building2, LayoutDashboard, Library, Search, Shield } from "lucide-react";

export const navItems = [
  { href: "/", label: "首页", icon: LayoutDashboard },
  { href: "/marketplace", label: "数据市场", icon: Search },
  { href: "/seller", label: "供应商后台", icon: Building2 },
  { href: "/buyer", label: "买家控制台", icon: Library },
  { href: "/ops", label: "平台运营", icon: Shield },
] as const;
