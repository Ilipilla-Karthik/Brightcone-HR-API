import React from 'react';
import {
  Menu,
  Search,
  Sun,
  Moon,
  Bell,
  Grid,
  ChevronRight,
  LogOut,
} from 'lucide-react';
import type { ActiveTab } from '../../types';

interface HeaderProps {
  onOpenMobileMenu: () => void;
  activeTab: ActiveTab;
  darkMode: boolean;
  setDarkMode: (val: boolean) => void;
  isBackendConnected: boolean | null;
  onRefreshConnection: () => void;
  searchQuery: string;
  setSearchQuery: (query: string) => void;
}

export const Header: React.FC<HeaderProps> = ({
  onOpenMobileMenu,
  activeTab,
  darkMode,
  setDarkMode,
  searchQuery,
  setSearchQuery,
}) => {
  const getBreadcrumbLabel = () => {
    switch (activeTab) {
      case 'weekly-roster':
        return 'Weekly roster';
      case 'daily-attendance':
        return 'Daily attendance';
      case 'shifts':
        return 'Attendance & shifts';
      case 'holidays':
        return 'Holiday calendar';
      case 'leave-types':
        return 'Leave requests';
      case 'leave-quotas':
        return 'Leave balance';
    }
  };

  return (
    <header className="sticky top-0 z-30 flex items-center justify-between gap-4 px-4 lg:px-8 py-3.5 bg-[#060D1A]/90 backdrop-blur-md border-b border-[#152347] transition-colors text-[#C9D3EE]">
      {/* Left: Mobile Toggle + Breadcrumbs */}
      <div className="flex items-center gap-3">
        <button
          onClick={onOpenMobileMenu}
          className="lg:hidden p-2 rounded-xl border border-[#152347] bg-[#0E1A38] text-[#8D9BB8] hover:text-white"
          aria-label="Open navigation menu"
        >
          <Menu className="w-5 h-5" />
        </button>

        {/* Breadcrumb Navigation matching screenshot */}
        <div className="flex items-center gap-2 text-xs font-medium">
          <div className="p-1.5 rounded-lg bg-[#0E1A38] border border-[#152347] text-[#8D9BB8]">
            <Grid className="w-4 h-4" />
          </div>
          <span className="text-[#8D9BB8] hover:text-white transition-colors cursor-pointer">
            Shift
          </span>
          <ChevronRight className="w-3.5 h-3.5 text-[#54658B]" />
          <span className="font-semibold text-white">
            {getBreadcrumbLabel()}
          </span>
        </div>
      </div>

      {/* Center: Global Search Bar */}
      <div className="flex-1 max-w-md hidden md:flex items-center">
        <div className="relative w-full flex items-center">
          <Search className="w-4 h-4 text-[#54658B] absolute left-3.5 pointer-events-none" />
          <input
            type="text"
            placeholder="Search people, pages or actions"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="w-full pl-10 pr-16 py-2 rounded-xl text-xs bg-[#091226] border border-[#18274A] text-white placeholder-[#54658B] focus:outline-hidden focus:border-[#0B6FDB] transition-all"
          />
          <span className="absolute right-3 px-1.5 py-0.5 rounded-md text-[10px] font-semibold text-[#54658B] bg-[#0E1A38] border border-[#18274A]">
            Ctrl K
          </span>
        </div>
      </div>

      {/* Right Tools & User Profile */}
      <div className="flex items-center gap-3">
        {/* Theme Toggle */}
        <button
          onClick={() => setDarkMode(!darkMode)}
          className="p-2 rounded-xl border border-[#152347] bg-[#091226] text-[#8D9BB8] hover:text-white transition-colors cursor-pointer"
          title="Toggle Theme"
        >
          {darkMode ? <Sun className="w-4 h-4 text-amber-400" /> : <Moon className="w-4 h-4 text-[#8D9BB8]" />}
        </button>

        {/* Notification Bell */}
        <button
          className="relative p-2 rounded-xl border border-[#152347] bg-[#091226] text-[#8D9BB8] hover:text-white transition-colors cursor-pointer"
          title="Notifications"
        >
          <Bell className="w-4 h-4" />
          <span className="absolute -top-1 -right-1 w-4 h-4 rounded-full text-[10px] font-bold bg-[#EF4444] text-white flex items-center justify-center">
            4
          </span>
        </button>

        {/* User Profile Pill */}
        <div className="flex items-center gap-2.5 pl-2 border-l border-[#152347]">
          <div className="w-8 h-8 rounded-full bg-[#722ED1] flex items-center justify-center text-white font-bold text-xs shadow-xs shrink-0">
            AR
          </div>
          <div className="hidden xl:block text-left text-xs leading-tight">
            <span className="font-semibold block text-white">Ananya Rao</span>
            <span className="text-[11px] text-[#647496]">People Ops Lead</span>
          </div>
          <button
            title="Exit"
            className="hidden sm:block p-1 text-[#647496] hover:text-white transition-colors cursor-pointer ml-1"
          >
            <LogOut className="w-3.5 h-3.5" />
          </button>
        </div>
      </div>
    </header>
  );
};
