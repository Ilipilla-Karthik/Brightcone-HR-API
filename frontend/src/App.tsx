import { useState, useEffect, useCallback } from 'react';
import type { ActiveTab, Shift, Holiday, LeaveType, LeaveQuota } from './types';
import {
  api,
  INITIAL_SHIFTS,
  INITIAL_HOLIDAYS,
  INITIAL_LEAVE_TYPES,
  INITIAL_LEAVE_QUOTAS,
} from './services/api';
import { Sidebar } from './components/layout/Sidebar';
import { Header } from './components/layout/Header';
import { ApiStatusBanner } from './components/common/ApiStatusBanner';
import { ShiftsPage } from './pages/ShiftsPage';
import { HolidaysPage } from './pages/HolidaysPage';
import { LeaveTypesPage } from './pages/LeaveTypesPage';
import { LeaveQuotasPage } from './pages/LeaveQuotasPage';
import { WeeklyRosterPage } from './pages/WeeklyRosterPage';
import { DailyAttendancePage } from './pages/DailyAttendancePage';
import { LeaveCalculatorPage } from './pages/LeaveCalculatorPage';
import { CheckCircle2, X } from 'lucide-react';

export function App() {
  const [activeTab, setActiveTab] = useState<ActiveTab>('weekly-roster');
  const [darkMode, setDarkMode] = useState<boolean>(true);
  const [isMobileMenuOpen, setIsMobileMenuOpen] = useState<boolean>(false);
  const [searchQuery, setSearchQuery] = useState<string>('');

  // Live Backend Connection & Feedback State
  const [isBackendConnected, setIsBackendConnected] = useState<boolean | null>(null);
  const [apiError, setApiError] = useState<string | null>(null);
  const [successMsg, setSuccessMsg] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState<boolean>(false);

  // Master Data Collections
  const [shifts, setShifts] = useState<Shift[]>(INITIAL_SHIFTS);
  const [holidays, setHolidays] = useState<Holiday[]>(INITIAL_HOLIDAYS);
  const [leaveTypes, setLeaveTypes] = useState<LeaveType[]>(INITIAL_LEAVE_TYPES);
  const [leaveQuotas, setLeaveQuotas] = useState<LeaveQuota[]>(INITIAL_LEAVE_QUOTAS);

  // Sync dark mode class on document element
  useEffect(() => {
    if (darkMode) {
      document.documentElement.classList.add('dark');
    } else {
      document.documentElement.classList.remove('dark');
    }
  }, [darkMode]);

  const showSuccess = (msg: string) => {
    setSuccessMsg(msg);
    setTimeout(() => {
      setSuccessMsg((current) => (current === msg ? null : current));
    }, 4500);
  };

  // Refresh lists from live backend
  const refreshShifts = useCallback(async () => {
    const res = await api.fetchShifts();
    if (res.data) setShifts(res.data);
  }, []);

  const refreshHolidays = useCallback(async () => {
    const res = await api.fetchHolidays();
    if (res.data) setHolidays(res.data);
  }, []);

  const refreshLeaveTypes = useCallback(async () => {
    const res = await api.fetchLeaveTypes();
    if (res.data) setLeaveTypes(res.data);
  }, []);

  const refreshLeaveQuotas = useCallback(async () => {
    const res = await api.fetchLeaveQuotas();
    if (res.data) setLeaveQuotas(res.data);
  }, []);

  // Check Backend Connection on mount and when requested
  const checkBackendStatus = useCallback(async () => {
    setIsLoading(true);
    try {
      const isLive = await api.checkHealth();
      setIsBackendConnected(isLive);

      if (isLive) {
        const [shiftsRes, holidaysRes, ltRes, lqRes] = await Promise.all([
          api.fetchShifts(),
          api.fetchHolidays(),
          api.fetchLeaveTypes(),
          api.fetchLeaveQuotas(),
        ]);

        if (shiftsRes.data) setShifts(shiftsRes.data);
        if (holidaysRes.data) setHolidays(holidaysRes.data);
        if (ltRes.data) setLeaveTypes(ltRes.data);
        if (lqRes.data) setLeaveQuotas(lqRes.data);
        setApiError(null);
      } else {
        setApiError('Connection refused to localhost:5150 (backend server not started)');
      }
    } catch (err: unknown) {
      setIsBackendConnected(false);
      setApiError(err instanceof Error ? err.message : 'Backend unreachable');
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    checkBackendStatus();
  }, [checkBackendStatus]);

  // =========================================================================
  // Shifts Handlers (Create, Update, Delete)
  // =========================================================================

  const handleAddShift = async (
    newShift: Omit<Shift, 'id' | 'pid' | 'created_at' | 'updated_at'>
  ): Promise<boolean> => {
    if (isBackendConnected) {
      try {
        await api.createShift(newShift);
        await refreshShifts();
        setApiError(null);
        showSuccess(`Shift "${newShift.name}" (${newShift.code}) saved to PostgreSQL.`);
        return true;
      } catch (err) {
        const msg = err instanceof Error ? err.message : 'Failed to create shift';
        setApiError(msg);
        throw new Error(msg);
      }
    }
    return false;
  };

  const handleUpdateShift = async (
    id: number,
    updatedFields: Partial<Shift>
  ): Promise<boolean> => {
    if (isBackendConnected) {
      try {
        await api.updateShift(id, updatedFields);
        await refreshShifts();
        setApiError(null);
        showSuccess('Shift updated in PostgreSQL.');
        return true;
      } catch (err) {
        const msg = err instanceof Error ? err.message : 'Failed to update shift';
        setApiError(msg);
        throw new Error(msg);
      }
    }
    return false;
  };

  const handleDeleteShift = async (id: number, name: string): Promise<boolean> => {
    if (isBackendConnected) {
      try {
        await api.deleteShift(id);
        await refreshShifts();
        setApiError(null);
        showSuccess(`Shift "${name}" deleted from PostgreSQL.`);
        return true;
      } catch (err) {
        const msg = err instanceof Error ? err.message : 'Failed to delete shift';
        setApiError(msg);
        throw new Error(msg);
      }
    }
    return false;
  };

  // =========================================================================
  // Holidays Handlers (Create, Update, Delete)
  // =========================================================================

  const handleAddHoliday = async (
    newHoliday: Omit<Holiday, 'id' | 'pid' | 'created_at' | 'updated_at'>
  ): Promise<boolean> => {
    if (isBackendConnected) {
      try {
        await api.createHoliday(newHoliday);
        await refreshHolidays();
        setApiError(null);
        showSuccess(`Holiday "${newHoliday.name}" saved to PostgreSQL.`);
        return true;
      } catch (err) {
        const msg = err instanceof Error ? err.message : 'Failed to create holiday';
        setApiError(msg);
        throw new Error(msg);
      }
    }
    return false;
  };

  const handleUpdateHoliday = async (
    id: number,
    updatedFields: Partial<Holiday>
  ): Promise<boolean> => {
    if (isBackendConnected) {
      try {
        await api.updateHoliday(id, updatedFields);
        await refreshHolidays();
        setApiError(null);
        showSuccess('Holiday updated in PostgreSQL.');
        return true;
      } catch (err) {
        const msg = err instanceof Error ? err.message : 'Failed to update holiday';
        setApiError(msg);
        throw new Error(msg);
      }
    }
    return false;
  };

  const handleDeleteHoliday = async (id: number, name: string): Promise<boolean> => {
    if (isBackendConnected) {
      try {
        await api.deleteHoliday(id);
        await refreshHolidays();
        setApiError(null);
        showSuccess(`Holiday "${name}" deleted from PostgreSQL.`);
        return true;
      } catch (err) {
        const msg = err instanceof Error ? err.message : 'Failed to delete holiday';
        setApiError(msg);
        throw new Error(msg);
      }
    }
    return false;
  };

  // =========================================================================
  // Leave Types Handlers (Create, Update, Delete)
  // =========================================================================

  const handleAddLeaveType = async (
    newLt: Omit<LeaveType, 'id' | 'pid' | 'created_at' | 'updated_at'>
  ): Promise<boolean> => {
    if (isBackendConnected) {
      try {
        await api.createLeaveType(newLt);
        await refreshLeaveTypes();
        setApiError(null);
        showSuccess(`Leave Type "${newLt.name}" (${newLt.code}) saved to PostgreSQL.`);
        return true;
      } catch (err) {
        const msg = err instanceof Error ? err.message : 'Failed to create leave type';
        setApiError(msg);
        throw new Error(msg);
      }
    }
    return false;
  };

  const handleUpdateLeaveType = async (
    id: number,
    updatedFields: Partial<LeaveType>
  ): Promise<boolean> => {
    if (isBackendConnected) {
      try {
        await api.updateLeaveType(id, updatedFields);
        await refreshLeaveTypes();
        await refreshLeaveQuotas();
        setApiError(null);
        showSuccess('Leave type updated in PostgreSQL.');
        return true;
      } catch (err) {
        const msg = err instanceof Error ? err.message : 'Failed to update leave type';
        setApiError(msg);
        throw new Error(msg);
      }
    }
    return false;
  };

  const handleDeleteLeaveType = async (id: number, name: string): Promise<boolean> => {
    if (isBackendConnected) {
      try {
        await api.deleteLeaveType(id);
        await refreshLeaveTypes();
        setApiError(null);
        showSuccess(`Leave category "${name}" deleted from PostgreSQL.`);
        return true;
      } catch (err) {
        const msg = err instanceof Error ? err.message : 'Failed to delete leave type';
        setApiError(msg);
        throw new Error(msg);
      }
    }
    return false;
  };

  // =========================================================================
  // Leave Quotas Handlers (Create, Update, Delete)
  // =========================================================================

  const handleAddLeaveQuota = async (
    newQuota: Omit<LeaveQuota, 'id' | 'pid' | 'created_at' | 'updated_at'>
  ): Promise<boolean> => {
    if (isBackendConnected) {
      try {
        await api.createLeaveQuota(newQuota);
        await refreshLeaveQuotas();
        setApiError(null);
        showSuccess('Leave quota rule saved to PostgreSQL.');
        return true;
      } catch (err) {
        const msg = err instanceof Error ? err.message : 'Failed to create quota allocation';
        setApiError(msg);
        throw new Error(msg);
      }
    }
    return false;
  };

  const handleUpdateLeaveQuota = async (
    id: number,
    updatedFields: Partial<LeaveQuota>
  ): Promise<boolean> => {
    if (isBackendConnected) {
      try {
        await api.updateLeaveQuota(id, updatedFields);
        await refreshLeaveQuotas();
        setApiError(null);
        showSuccess('Leave quota updated in PostgreSQL.');
        return true;
      } catch (err) {
        const msg = err instanceof Error ? err.message : 'Failed to update leave quota';
        setApiError(msg);
        throw new Error(msg);
      }
    }
    return false;
  };

  const handleDeleteLeaveQuota = async (id: number, label: string): Promise<boolean> => {
    if (isBackendConnected) {
      try {
        await api.deleteLeaveQuota(id);
        await refreshLeaveQuotas();
        setApiError(null);
        showSuccess(`Quota for ${label} deleted from PostgreSQL.`);
        return true;
      } catch (err) {
        const msg = err instanceof Error ? err.message : 'Failed to delete leave quota';
        setApiError(msg);
        throw new Error(msg);
      }
    }
    return false;
  };

  return (
    <div className="min-h-screen bg-[#F8FAFD] dark:bg-[#060D1A] text-[#0E1730] dark:text-[#E8EDFB] flex flex-col font-['Inter'] antialiased">
      {/* Toast Success Notification */}
      {successMsg && (
        <div className="fixed top-5 right-5 z-50 flex items-center gap-3 px-4 py-3 rounded-xl bg-[#E3F6EE] dark:bg-[#0E3326] border border-[#15A36B]/40 text-[#15A36B] shadow-lg text-xs font-semibold animate-in fade-in slide-in-from-top-2">
          <CheckCircle2 className="w-4 h-4 flex-none" />
          <span>{successMsg}</span>
          <button
            onClick={() => setSuccessMsg(null)}
            className="p-1 hover:bg-[#15A36B]/10 rounded-lg transition-colors cursor-pointer"
          >
            <X className="w-3.5 h-3.5" />
          </button>
        </div>
      )}

      {/* Sidebar Navigation */}
      <Sidebar
        activeTab={activeTab}
        setActiveTab={setActiveTab}
        isOpen={isMobileMenuOpen}
        onClose={() => setIsMobileMenuOpen(false)}
        shiftsCount={shifts.length}
        holidaysCount={holidays.length}
        leaveTypesCount={leaveTypes.length}
        leaveQuotasCount={leaveQuotas.length}
      />

      {/* Main Content Area */}
      <div className="flex-1 flex flex-col min-w-0 lg:pl-[304px]">
        {/* Header */}
        <Header
          onOpenMobileMenu={() => setIsMobileMenuOpen(true)}
          activeTab={activeTab}
          darkMode={darkMode}
          setDarkMode={setDarkMode}
          isBackendConnected={isBackendConnected}
          onRefreshConnection={checkBackendStatus}
          searchQuery={searchQuery}
          setSearchQuery={setSearchQuery}
        />

        {/* View Content */}
        <main className="flex-1 p-4 lg:p-8 max-w-7xl w-full mx-auto">
          {/* Status Banner */}
          <ApiStatusBanner
            isBackendConnected={isBackendConnected}
            onRefresh={checkBackendStatus}
            error={apiError}
          />

          {isLoading ? (
            <div className="flex items-center justify-center p-12 text-[#5B6787] dark:text-[#8D97B0]">
              <div className="flex items-center gap-3 text-xs">
                <div className="w-4 h-4 border-2 border-[#0B6FDB] border-t-transparent rounded-full animate-spin" />
                <span>Synchronizing PostgreSQL tables...</span>
              </div>
            </div>
          ) : (
            <>
              {activeTab === 'weekly-roster' && (
                <WeeklyRosterPage
                  shifts={shifts}
                  searchQuery={searchQuery}
                  showSuccessToast={showSuccess}
                />
              )}

              {activeTab === 'daily-attendance' && (
                <DailyAttendancePage
                  showSuccessToast={showSuccess}
                  onNavigateToEmployees={() => setActiveTab('weekly-roster')}
                />
              )}

              {activeTab === 'shifts' && (
                <ShiftsPage
                  shifts={shifts}
                  onAddShift={handleAddShift}
                  onUpdateShift={handleUpdateShift}
                  onDeleteShift={handleDeleteShift}
                  searchQuery={searchQuery}
                />
              )}

              {activeTab === 'holidays' && (
                <HolidaysPage
                  holidays={holidays}
                  onAddHoliday={handleAddHoliday}
                  onUpdateHoliday={handleUpdateHoliday}
                  onDeleteHoliday={handleDeleteHoliday}
                  searchQuery={searchQuery}
                />
              )}

              {activeTab === 'leave-types' && (
                <LeaveTypesPage
                  leaveTypes={leaveTypes}
                  onAddLeaveType={handleAddLeaveType}
                  onUpdateLeaveType={handleUpdateLeaveType}
                  onDeleteLeaveType={handleDeleteLeaveType}
                  searchQuery={searchQuery}
                />
              )}

              {activeTab === 'leave-quotas' && (
                <LeaveQuotasPage
                  leaveQuotas={leaveQuotas}
                  leaveTypes={leaveTypes}
                  onAddQuota={handleAddLeaveQuota}
                  onUpdateQuota={handleUpdateLeaveQuota}
                  onDeleteQuota={handleDeleteLeaveQuota}
                  searchQuery={searchQuery}
                />
              )}

              {activeTab === 'leave-calculator' && (
                <LeaveCalculatorPage />
              )}
            </>
          )}
        </main>

        {/* Footer */}
        <footer className="py-4 px-8 border-t border-[#E3E8F2] dark:border-[#1D2B52] text-xs text-[#8D97B0] flex flex-col sm:flex-row items-center justify-between gap-2">
          <span>Brightcone People Plus · Attendance & Leave Module</span>
          <span>PostgreSQL Schema · Rust + Loco Backend · Live Persisted Storage</span>
        </footer>
      </div>
    </div>
  );
}

export default App;
