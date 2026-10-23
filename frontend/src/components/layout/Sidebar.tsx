import React from 'react';
import {
  Calendar,
  Clock,
  FileText,
  PieChart,
  X,
  Users,
  Briefcase,
  CreditCard,
  Receipt,
  FileSignature,
  MessageSquare,
  GraduationCap,
  Sparkles,
  Home,
  ArrowLeftRight,
  LogOut,
  LogIn,
  Calculator,
} from 'lucide-react';
import type { ActiveTab } from '../../types';

interface SidebarProps {
  activeTab: ActiveTab;
  setActiveTab: (tab: ActiveTab) => void;
  isOpen: boolean;
  onClose: () => void;
  shiftsCount: number;
  holidaysCount: number;
  leaveTypesCount: number;
  leaveQuotasCount: number;
}

export const Sidebar: React.FC<SidebarProps> = ({
  activeTab,
  setActiveTab,
  isOpen,
  onClose,
  shiftsCount,
  holidaysCount,
  leaveTypesCount,
  leaveQuotasCount,
}) => {
  // Far-left slim rail modules matching screenshot
  const railModules = [
    { id: 'people', icon: Users, label: 'People', badge: 5 },
    { id: 'recruit', icon: Briefcase, label: 'Recruit' },
    { id: 'payroll', icon: CreditCard, label: 'Payroll' },
    { id: 'expense', icon: Receipt, label: 'Expense', badge: 2 },
    { id: 'sign', icon: FileSignature, label: 'Sign' },
    { id: 'connect', icon: MessageSquare, label: 'Connect' },
    { id: 'learn', icon: GraduationCap, label: 'Learn' },
    { id: 'shift', icon: Calendar, label: 'Shift', badge: 3, isActive: true },
    { id: 'plus', icon: Sparkles, label: 'People Plus', badge: 10 },
  ];

  return (
    <>
      {/* Mobile Backdrop */}
      {isOpen && (
        <div
          className="fixed inset-0 bg-black/60 z-40 lg:hidden backdrop-blur-xs"
          onClick={onClose}
        />
      )}

      {/* Main Dual Sidebar Container */}
      <aside
        className={`fixed top-0 bottom-0 left-0 z-50 flex transition-transform duration-200 ease-in-out lg:translate-x-0 ${
          isOpen ? 'translate-x-0' : '-translate-x-full lg:translate-x-0'
        }`}
      >
        {/* Far-Left Slim Icon Rail */}
        <div className="w-16 bg-[#040A15] border-r border-[#101C38] flex flex-col items-center py-4 justify-between shrink-0 select-none">
          {/* App Top Logo */}
          <div className="w-9 h-9 rounded-xl bg-gradient-to-tr from-[#0B6FDB] to-[#0BA6CC] flex items-center justify-center font-bold text-white text-base shadow-lg shadow-[#0B6FDB]/30 mb-5">
            ▲
          </div>

          {/* Module Icons List */}
          <div className="flex-1 w-full space-y-3.5 flex flex-col items-center overflow-y-auto no-scrollbar">
            {railModules.map((m) => {
              const Icon = m.icon;
              return (
                <button
                  key={m.id}
                  title={m.label}
                  className={`relative p-2 rounded-xl transition-all cursor-pointer group flex flex-col items-center ${
                    m.isActive
                      ? 'bg-[#0B6FDB]/20 text-[#0BA6CC] border border-[#0BA6CC]/40 shadow-xs'
                      : 'text-[#647496] hover:bg-white/5 hover:text-[#C9D3EE]'
                  }`}
                >
                  <Icon className="w-5 h-5" />
                  {m.badge && (
                    <span className="absolute -top-1 -right-1 px-1 min-w-4 h-4 rounded-full text-[9px] font-extrabold bg-[#EF4444] text-white flex items-center justify-center">
                      {m.badge}
                    </span>
                  )}
                  <span className="text-[9px] font-medium tracking-tight mt-0.5 truncate max-w-12 text-center">
                    {m.label}
                  </span>
                </button>
              );
            })}
          </div>
        </div>

        {/* Sub-Sidebar (Shift & Attendance Menu) */}
        <div className="w-60 bg-[#070F22] text-[#C9D3EE] border-r border-[#152347] flex flex-col p-4">
          {/* Header */}
          <div className="flex items-center justify-between pb-4 px-1 border-b border-[#152347]">
            <div className="flex items-center gap-3">
              <div className="w-10 h-10 rounded-xl bg-[#0B6FDB]/20 border border-[#0B6FDB]/40 flex items-center justify-center text-[#0BA6CC] shadow-inner">
                <Calendar className="w-5 h-5" />
              </div>
              <div>
                <b className="font-['Sora'] text-white text-base font-bold block leading-tight">
                  Shift
                </b>
                <span className="text-[#647496] text-xs font-medium block">
                  HR Admin
                </span>
              </div>
            </div>
            <button
              onClick={onClose}
              className="lg:hidden p-1.5 rounded-lg hover:bg-white/10 text-[#8D9BB8] hover:text-white"
            >
              <X className="w-5 h-5" />
            </button>
          </div>

          {/* Navigation Sections */}
          <div className="flex-1 overflow-y-auto py-4 space-y-5 text-xs font-medium">
            {/* 1. OVERVIEW */}
            <div>
              <div className="text-[10px] font-bold uppercase tracking-wider text-[#54658B] px-2 mb-1.5">
                Overview
              </div>
              <button
                onClick={() => {
                  setActiveTab('weekly-roster');
                  onClose();
                }}
                className={`w-full flex items-center gap-2.5 px-3 py-2 rounded-xl transition-all cursor-pointer ${
                  activeTab === 'weekly-roster'
                    ? 'bg-[#0B6FDB]/20 text-white border border-[#0B6FDB]/40 font-semibold shadow-xs'
                    : 'text-[#8D9BB8] hover:bg-white/5 hover:text-white'
                }`}
              >
                <Calendar className={`w-4 h-4 ${activeTab === 'weekly-roster' ? 'text-[#0BA6CC]' : 'opacity-70'}`} />
                <span>Weekly roster</span>
              </button>
            </div>

            {/* 2. ATTENDANCE & LEAVE */}
            <div>
              <div className="text-[10px] font-bold uppercase tracking-wider text-[#54658B] px-2 mb-1.5">
                Attendance & Leave
              </div>
              <div className="space-y-0.5">
                <button
                  type="button"
                  onClick={() => {
                    setActiveTab('daily-attendance');
                    onClose();
                  }}
                  className={`w-full flex items-center gap-2.5 px-3 py-2 rounded-xl transition-all cursor-pointer text-left ${
                    activeTab === 'daily-attendance'
                      ? 'bg-[#0B6FDB]/20 text-white border border-[#0B6FDB]/40 font-semibold shadow-xs'
                      : 'text-[#8D9BB8] hover:bg-white/5 hover:text-white'
                  }`}
                >
                  <Clock className={`w-4 h-4 ${activeTab === 'daily-attendance' ? 'text-[#0BA6CC]' : 'opacity-70'}`} />
                  <span>Daily attendance</span>
                </button>

                <button
                  type="button"
                  className="w-full flex items-center gap-2.5 px-3 py-2 rounded-xl text-[#6D7D9F] hover:bg-white/5 hover:text-[#C9D3EE] transition-colors cursor-pointer text-left"
                >
                  <LogIn className="w-4 h-4 opacity-70" />
                  <span>Login & logout</span>
                </button>

                <button
                  onClick={() => {
                    setActiveTab('leave-types');
                    onClose();
                  }}
                  className={`w-full flex items-center justify-between px-3 py-2 rounded-xl transition-all cursor-pointer ${
                    activeTab === 'leave-types'
                      ? 'bg-[#0B6FDB]/20 text-white border border-[#0B6FDB]/40 font-semibold shadow-xs'
                      : 'text-[#8D9BB8] hover:bg-white/5 hover:text-white'
                  }`}
                >
                  <div className="flex items-center gap-2.5 truncate">
                    <FileText className={`w-4 h-4 ${activeTab === 'leave-types' ? 'text-[#0BA6CC]' : 'opacity-70'}`} />
                    <span>Leave requests</span>
                  </div>
                  <span className="text-[10px] px-1.5 py-0.5 rounded-full bg-[#142347] text-[#8D9BB8] font-bold">
                    {leaveTypesCount}
                  </span>
                </button>

                <button
                  onClick={() => {
                    setActiveTab('leave-quotas');
                    onClose();
                  }}
                  className={`w-full flex items-center justify-between px-3 py-2 rounded-xl transition-all cursor-pointer ${
                    activeTab === 'leave-quotas'
                      ? 'bg-[#0B6FDB]/20 text-white border border-[#0B6FDB]/40 font-semibold shadow-xs'
                      : 'text-[#8D9BB8] hover:bg-white/5 hover:text-white'
                  }`}
                >
                  <div className="flex items-center gap-2.5 truncate">
                    <PieChart className={`w-4 h-4 ${activeTab === 'leave-quotas' ? 'text-[#0BA6CC]' : 'opacity-70'}`} />
                    <span>Leave balance</span>
                  </div>
                  <span className="text-[10px] px-1.5 py-0.5 rounded-full bg-[#142347] text-[#8D9BB8] font-bold">
                    {leaveQuotasCount}
                  </span>
                </button>

                <button
                  onClick={() => {
                    setActiveTab('leave-calculator');
                    onClose();
                  }}
                  className={`w-full flex items-center justify-between px-3 py-2 rounded-xl transition-all cursor-pointer ${
                    activeTab === 'leave-calculator'
                      ? 'bg-[#0B6FDB]/20 text-white border border-[#0B6FDB]/40 font-semibold shadow-xs'
                      : 'text-[#8D9BB8] hover:bg-white/5 hover:text-white'
                  }`}
                >
                  <div className="flex items-center gap-2.5 truncate">
                    <Calculator className={`w-4 h-4 ${activeTab === 'leave-calculator' ? 'text-[#0BA6CC]' : 'opacity-70'}`} />
                    <span>Leave calculator</span>
                  </div>
                  <span className="text-[9px] px-1.5 py-0.5 rounded-full bg-[#0B6FDB]/20 text-[#0BA6CC] font-bold">
                    TIM-04
                  </span>
                </button>

                <button
                  type="button"
                  className="w-full flex items-center gap-2.5 px-3 py-2 rounded-xl text-[#6D7D9F] hover:bg-white/5 hover:text-[#C9D3EE] transition-colors cursor-pointer text-left"
                >
                  <Home className="w-4 h-4 opacity-70" />
                  <span>Work from home</span>
                </button>

                <button
                  type="button"
                  className="w-full flex items-center gap-2.5 px-3 py-2 rounded-xl text-[#6D7D9F] hover:bg-white/5 hover:text-[#C9D3EE] transition-colors cursor-pointer text-left"
                >
                  <ArrowLeftRight className="w-4 h-4 opacity-70" />
                  <span>Comp-off tracker</span>
                </button>

                <button
                  onClick={() => {
                    setActiveTab('holidays');
                    onClose();
                  }}
                  className={`w-full flex items-center justify-between px-3 py-2 rounded-xl transition-all cursor-pointer ${
                    activeTab === 'holidays'
                      ? 'bg-[#0B6FDB]/20 text-white border border-[#0B6FDB]/40 font-semibold shadow-xs'
                      : 'text-[#8D9BB8] hover:bg-white/5 hover:text-white'
                  }`}
                >
                  <div className="flex items-center gap-2.5 truncate">
                    <Calendar className={`w-4 h-4 ${activeTab === 'holidays' ? 'text-[#0BA6CC]' : 'opacity-70'}`} />
                    <span>Holiday calendar</span>
                  </div>
                  <span className="text-[10px] px-1.5 py-0.5 rounded-full bg-[#142347] text-[#8D9BB8] font-bold">
                    {holidaysCount}
                  </span>
                </button>
              </div>
            </div>

            {/* 3. MANAGE */}
            <div>
              <div className="text-[10px] font-bold uppercase tracking-wider text-[#54658B] px-2 mb-1.5">
                Manage
              </div>
              <div className="space-y-0.5">
                <button
                  type="button"
                  className="w-full flex items-center gap-2.5 px-3 py-2 rounded-xl text-[#6D7D9F] hover:bg-white/5 hover:text-[#C9D3EE] transition-colors cursor-pointer text-left"
                >
                  <ArrowLeftRight className="w-4 h-4 opacity-70" />
                  <span>Swap requests</span>
                </button>

                <button
                  onClick={() => {
                    setActiveTab('shifts');
                    onClose();
                  }}
                  className={`w-full flex items-center justify-between px-3 py-2 rounded-xl transition-all cursor-pointer ${
                    activeTab === 'shifts'
                      ? 'bg-[#0B6FDB]/20 text-white border border-[#0B6FDB]/40 font-semibold shadow-xs'
                      : 'text-[#8D9BB8] hover:bg-white/5 hover:text-white'
                  }`}
                >
                  <div className="flex items-center gap-2.5 truncate">
                    <Clock className={`w-4 h-4 ${activeTab === 'shifts' ? 'text-[#0BA6CC]' : 'opacity-70'}`} />
                    <span>Attendance & shifts</span>
                  </div>
                  <span className="text-[10px] px-1.5 py-0.5 rounded-full bg-[#142347] text-[#8D9BB8] font-bold">
                    {shiftsCount}
                  </span>
                </button>
              </div>
            </div>
          </div>

          {/* Bottom Profile Card */}
          <div className="pt-3 border-t border-[#152347] flex items-center justify-between">
            <div className="flex items-center gap-2.5 min-w-0">
              <div className="w-8 h-8 rounded-full bg-[#722ED1] flex items-center justify-center font-bold text-xs text-white shadow-xs shrink-0">
                AR
              </div>
              <div className="min-w-0">
                <div className="font-semibold text-white truncate text-xs">
                  Ananya Rao
                </div>
                <div className="text-[10px] text-[#647496] truncate">
                  ananya.rao@b...
                </div>
              </div>
            </div>
            <button
              title="Logout"
              className="p-1 rounded-lg text-[#647496] hover:text-white hover:bg-white/5 transition-colors cursor-pointer"
            >
              <LogOut className="w-3.5 h-3.5" />
            </button>
          </div>
        </div>
      </aside>
    </>
  );
};
