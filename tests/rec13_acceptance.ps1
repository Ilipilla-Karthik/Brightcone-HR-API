# REC-13 Production Acceptance Test Suite: Learning Flow & Resource Management
# Comprehensive end-to-end verification across API, PostgreSQL, and Authorization

$ErrorActionPreference = "Stop"

$baseUrl = "http://127.0.0.1:5155/api/v1"

Write-Host "========================================================"
Write-Host "Starting REC-13 Production-Quality Acceptance Test Suite"
Write-Host "========================================================"

# 1. Authenticate as Admin (Ananya Rao)
$adminLoginBody = @{ email = "ananya.rao@brightcone.ai"; password = "Admin@123" } | ConvertTo-Json
$adminLoginRes = Invoke-RestMethod -Uri "$baseUrl/auth/login" -Method Post -Body $adminLoginBody -ContentType "application/json"
$adminToken = $adminLoginRes.token

# 2. Authenticate as Employee 1 (Rahul Varma, BC-1038)
$empLoginBody = @{ email = "rahul.varma@brightcone.ai"; password = "Welcome@123" } | ConvertTo-Json
$empLoginRes = Invoke-RestMethod -Uri "$baseUrl/auth/login" -Method Post -Body $empLoginBody -ContentType "application/json"
$empToken = $empLoginRes.token

# 3. Authenticate as Employee 2 (user2@example.com)
$emp2LoginBody = @{ email = "user2@example.com"; password = "12341234" } | ConvertTo-Json
$emp2LoginRes = Invoke-RestMethod -Uri "$baseUrl/auth/login" -Method Post -Body $emp2LoginBody -ContentType "application/json"
$emp2Token = $emp2LoginRes.token

$empHeaders = @{ Authorization = "Bearer $empToken" }
$emp2Headers = @{ Authorization = "Bearer $emp2Token" }
$adminHeaders = @{ Authorization = "Bearer $adminToken" }

$today = (Get-Date).ToString("yyyy-MM-dd")
$yesterday = (Get-Date).AddDays(-1).ToString("yyyy-MM-dd")
$tomorrow = (Get-Date).AddDays(1).ToString("yyyy-MM-dd")
$future = (Get-Date).AddDays(30).ToString("yyyy-MM-dd")

$passCount = 0
$failCount = 0

function Report-Scenario {
    param($Id, $Desc, $Pass, $Details)
    Write-Host "`n========================================================"
    Write-Host "TEST ${Id}: $Desc"
    if ($Pass) {
        Write-Host "STATUS: PASS" -ForegroundColor Green
        $script:passCount++
    } else {
        Write-Host "STATUS: FAIL" -ForegroundColor Red
        $script:failCount++
    }
    Write-Host "DETAILS: $Details"
    Write-Host "========================================================"
}

# ---------------------------------------------------------------------------
# Setup test courses
# ---------------------------------------------------------------------------

# Course 1: Mandatory, Past Due (Yesterday), with document resource attached
$c1Body = @{
    title = "REC13-C1 Past Due Mandatory"
    course_type = "Mandatory"
    duration = "30 min"
    audience = "Everyone"
    due_date = $yesterday
    content_type = "document"
    content_url = "https://company.example.com/materials/c1_compliance.pdf"
} | ConvertTo-Json
$c1 = Invoke-RestMethod -Uri "$baseUrl/learn/courses" -Method Post -Headers $adminHeaders -Body $c1Body -ContentType "application/json"

# Course 2: Mandatory, Due Today, with video resource attached
$c2Body = @{
    title = "REC13-C2 Due Today Mandatory"
    course_type = "Mandatory"
    duration = "45 min"
    audience = "Everyone"
    due_date = $today
    content_type = "video"
    content_url = "https://stream.example.com/materials/c2_lecture.mp4"
} | ConvertTo-Json
$c2 = Invoke-RestMethod -Uri "$baseUrl/learn/courses" -Method Post -Headers $adminHeaders -Body $c2Body -ContentType "application/json"

# Course 3: Mandatory, Future Due, with link resource attached
$c3Body = @{
    title = "REC13-C3 Future Due Mandatory"
    course_type = "Mandatory"
    duration = "60 min"
    audience = "Everyone"
    due_date = $future
    content_type = "link"
    content_url = "https://portal.example.com/materials/c3_portal"
} | ConvertTo-Json
$c3 = Invoke-RestMethod -Uri "$baseUrl/learn/courses" -Method Post -Headers $adminHeaders -Body $c3Body -ContentType "application/json"

# Course 4: Elective, Past Due, with link resource attached
$c4Body = @{
    title = "REC13-C4 Past Due Elective"
    course_type = "Elective"
    duration = "20 min"
    audience = "Everyone"
    due_date = $yesterday
    content_type = "link"
    content_url = "https://portal.example.com/materials/c4_elective"
} | ConvertTo-Json
$c4 = Invoke-RestMethod -Uri "$baseUrl/learn/courses" -Method Post -Headers $adminHeaders -Body $c4Body -ContentType "application/json"

# Course 5: Mandatory, No Resource attached (Honest unavailable state)
$c5Body = @{
    title = "REC13-C5 Resource-less Course"
    course_type = "Mandatory"
    duration = "15 min"
    audience = "Everyone"
    due_date = $null
    content_type = $null
    content_url = $null
} | ConvertTo-Json
$c5 = Invoke-RestMethod -Uri "$baseUrl/learn/courses" -Method Post -Headers $adminHeaders -Body $c5Body -ContentType "application/json"

# Retrieve employee assignments
$myAssignments = Invoke-RestMethod -Uri "$baseUrl/learn/my-learning" -Method Get -Headers $empHeaders

$a1 = $myAssignments | Where-Object { $_.course_id -eq $c1.id }
$a2 = $myAssignments | Where-Object { $_.course_id -eq $c2.id }
$a3 = $myAssignments | Where-Object { $_.course_id -eq $c3.id }
$a4 = $myAssignments | Where-Object { $_.course_id -eq $c4.id }
$a5 = $myAssignments | Where-Object { $_.course_id -eq $c5.id }

# ---------------------------------------------------------------------------
# SCENARIO 1: Mandatory incomplete course past due date is flagged overdue
# ---------------------------------------------------------------------------
$s1Pass = ($a1 -ne $null) -and ($a1.is_overdue -eq $true) -and ($a1.progress -lt 100)
Report-Scenario "SCENARIO_1" "Mandatory incomplete course past due date is flagged overdue" $s1Pass "Course: $($a1.course_title), Due: $($a1.due_date), Progress: $($a1.progress)%, is_overdue=$($a1.is_overdue)"

# ---------------------------------------------------------------------------
# SCENARIO 2: Mandatory course due today is NOT overdue under date-only policy
# ---------------------------------------------------------------------------
$s2Pass = ($a2 -ne $null) -and ($a2.is_overdue -eq $false)
Report-Scenario "SCENARIO_2" "Mandatory course due today remains due through the calendar day (not overdue)" $s2Pass "Course: $($a2.course_title), Due: $($a2.due_date), Today: $today, is_overdue=$($a2.is_overdue)"

# ---------------------------------------------------------------------------
# SCENARIO 3: Mandatory course with future deadline is NOT overdue
# ---------------------------------------------------------------------------
$s3Pass = ($a3 -ne $null) -and ($a3.is_overdue -eq $false)
Report-Scenario "SCENARIO_3" "Mandatory course with future deadline is not overdue" $s3Pass "Course: $($a3.course_title), Due: $($a3.due_date), is_overdue=$($a3.is_overdue)"

# ---------------------------------------------------------------------------
# SCENARIO 4: Elective course with past due date does NOT get mandatory overdue flag
# ---------------------------------------------------------------------------
$s4Pass = ($a4 -ne $null) -and ($a4.is_overdue -eq $false)
Report-Scenario "SCENARIO_4" "Elective course with past due date is not flagged overdue" $s4Pass "Course: $($a4.course_title), Type: $($a4.course_type), Due: $($a4.due_date), is_overdue=$($a4.is_overdue)"

# ---------------------------------------------------------------------------
# SCENARIO 5: Course without a due date is NOT overdue
# ---------------------------------------------------------------------------
$s5Pass = ($a5 -ne $null) -and ($a5.is_overdue -eq $false)
Report-Scenario "SCENARIO_5" "Course without a due date is not overdue" $s5Pass "Course: $($a5.course_title), Due: $($a5.due_date), is_overdue=$($a5.is_overdue)"

# ---------------------------------------------------------------------------
# SCENARIO 6: Assigned employee receives authoritative resource metadata
# ---------------------------------------------------------------------------
$s6Pass = ($a1.content_type -eq "document") -and ($a1.content_url -eq "https://company.example.com/materials/c1_compliance.pdf") -and
          ($a2.content_type -eq "video") -and ($a2.content_url -eq "https://stream.example.com/materials/c2_lecture.mp4")
Report-Scenario "SCENARIO_6" "Assigned employee retrieves correct resource type and HTTPS URL through authenticated assignment flow" $s6Pass "C1: $($a1.content_type) -> $($a1.content_url), C2: $($a2.content_type) -> $($a2.content_url)"

# ---------------------------------------------------------------------------
# SCENARIO 7: Resource-less course shows honest unavailable status (null resource fields)
# ---------------------------------------------------------------------------
$s7Pass = ($a5.content_type -eq $null) -and ($a5.content_url -eq $null)
Report-Scenario "SCENARIO_7" "Resource-less course has empty resource fields" $s7Pass "C5 content_type=$($a5.content_type), content_url=$($a5.content_url)"

# ---------------------------------------------------------------------------
# SCENARIO 8: Attempting to complete a course WITHOUT learning material is strictly rejected (HTTP 400)
# ---------------------------------------------------------------------------
$c5CompletionBlocked = $false
$c5Code = 0
try {
    $c5Update = @{ progress = 100 } | ConvertTo-Json
    Invoke-RestMethod -Uri "$baseUrl/learn/assignments/$($a5.assignment_id)/progress" -Method Post -Headers $empHeaders -Body $c5Update -ContentType "application/json"
} catch {
    $c5Code = $_.Exception.Response.StatusCode.value__
    if ($c5Code -eq 400) {
        $c5CompletionBlocked = $true
    }
}
Report-Scenario "SCENARIO_8" "Completion of course without attached material is strictly rejected with HTTP 400" $c5CompletionBlocked "Rejected with status code: $c5Code"

# ---------------------------------------------------------------------------
# SCENARIO 9: Course with material allows employee self-reported completion (removes overdue)
# ---------------------------------------------------------------------------
$updateBody = @{ progress = 100 } | ConvertTo-Json
$updatedA1 = Invoke-RestMethod -Uri "$baseUrl/learn/assignments/$($a1.assignment_id)/progress" -Method Post -Headers $empHeaders -Body $updateBody -ContentType "application/json"
$s9Pass = ($updatedA1.progress -eq 100) -and ($updatedA1.status -eq "Completed") -and ($updatedA1.is_overdue -eq $false) -and ($updatedA1.assignment_id -eq $a1.assignment_id)
Report-Scenario "SCENARIO_9" "Completed mandatory course with material succeeds and clears overdue badge" $s9Pass "Updated progress: $($updatedA1.progress)%, Status: $($updatedA1.status), is_overdue=$($updatedA1.is_overdue)"

# ---------------------------------------------------------------------------
# SCENARIO 10: Completion persists in PostgreSQL across fresh requests
# ---------------------------------------------------------------------------
$refetched = Invoke-RestMethod -Uri "$baseUrl/learn/my-learning" -Method Get -Headers $empHeaders
$refetchedA1 = $refetched | Where-Object { $_.course_id -eq $c1.id }
$s10Pass = ($refetchedA1.progress -eq 100) -and ($refetchedA1.status -eq "Completed") -and ($refetchedA1.is_overdue -eq $false) -and ($refetchedA1.completed_at -ne $null)
Report-Scenario "SCENARIO_10" "Self-reported completion persists in PostgreSQL across session requests" $s10Pass "DB status: $($refetchedA1.status), completed_at=$($refetchedA1.completed_at), is_overdue=$($refetchedA1.is_overdue)"

# ---------------------------------------------------------------------------
# SCENARIO 11: Single assignment retrieval endpoint verifies employee ownership
# ---------------------------------------------------------------------------
$singleAssign = Invoke-RestMethod -Uri "$baseUrl/learn/assignments/$($a1.assignment_id)" -Method Get -Headers $empHeaders
$s11Pass = ($singleAssign.assignment_id -eq $a1.assignment_id) -and ($singleAssign.content_url -eq "https://company.example.com/materials/c1_compliance.pdf")
Report-Scenario "SCENARIO_11" "Assigned employee can retrieve their assignment directly via /assignments/{id}" $s11Pass "Retrieved: $($singleAssign.assignment_id), Material: $($singleAssign.content_url)"

# ---------------------------------------------------------------------------
# SCENARIO 12: Another employee CANNOT retrieve a protected assignment (Ownership enforcement)
# ---------------------------------------------------------------------------
$crossRetrievalBlocked = $false
$crossGetCode = 0
try {
    # Priya attempts to access Rahul's assignment directly
    Invoke-RestMethod -Uri "$baseUrl/learn/assignments/$($a1.assignment_id)" -Method Get -Headers $emp2Headers
} catch {
    $crossGetCode = $_.Exception.Response.StatusCode.value__
    if ($crossGetCode -eq 401 -or $crossGetCode -eq 403) {
        $crossRetrievalBlocked = $true
    }
}
Report-Scenario "SCENARIO_12" "Another employee cannot access an assignment they do not own (HTTP 401/403)" $crossRetrievalBlocked "Status code: $crossGetCode"

# ---------------------------------------------------------------------------
# SCENARIO 13: Another employee CANNOT modify assignment progress
# ---------------------------------------------------------------------------
$tamperAttemptBlocked = $false
$tamperCode = 0
try {
    Invoke-RestMethod -Uri "$baseUrl/learn/assignments/$($a1.assignment_id)/progress" -Method Post -Headers $emp2Headers -Body $updateBody -ContentType "application/json"
} catch {
    $tamperCode = $_.Exception.Response.StatusCode.value__
    if ($tamperCode -eq 401 -or $tamperCode -eq 403) {
        $tamperAttemptBlocked = $true
    }
}
Report-Scenario "SCENARIO_13" "Cross-employee assignment progress modification is strictly rejected" $tamperAttemptBlocked "Tamper attempt status code: $tamperCode"

# ---------------------------------------------------------------------------
# SCENARIO 14: Admin can edit course resource (updates persist)
# ---------------------------------------------------------------------------
$editBody = @{
    title = "REC13-C1 Past Due Mandatory (Updated)"
    course_type = "Mandatory"
    duration = "35 min"
    audience = "Everyone"
    due_date = $yesterday
    content_type = "video"
    content_url = "https://company.example.com/materials/c1_updated_video.mp4"
} | ConvertTo-Json
$editedC1 = Invoke-RestMethod -Uri "$baseUrl/learn/courses/$($c1.id)" -Method Put -Headers $adminHeaders -Body $editBody -ContentType "application/json"
$s14Pass = ($editedC1.content_type -eq "video") -and ($editedC1.content_url -eq "https://company.example.com/materials/c1_updated_video.mp4") -and ($editedC1.title -eq "REC13-C1 Past Due Mandatory (Updated)")
Report-Scenario "SCENARIO_14" "Admin edits course resource and updates persist" $s14Pass "New type: $($editedC1.content_type), New URL: $($editedC1.content_url)"

# ---------------------------------------------------------------------------
# SCENARIO 15: Resource edits do NOT silently reset previous employee completions
# ---------------------------------------------------------------------------
$checkRefreshedA1 = Invoke-RestMethod -Uri "$baseUrl/learn/assignments/$($a1.assignment_id)" -Method Get -Headers $empHeaders
$s15Pass = ($checkRefreshedA1.progress -eq 100) -and ($checkRefreshedA1.status -eq "Completed")
Report-Scenario "SCENARIO_15" "Resource edits do not reset previous completions" $s15Pass "Completion retained: progress=$($checkRefreshedA1.progress)%, status=$($checkRefreshedA1.status)"

# ---------------------------------------------------------------------------
# SCENARIO 16: Admin can explicitly clear a resource (null/empty values)
# ---------------------------------------------------------------------------
$clearBody = @{
    title = "REC13-C1 Past Due Mandatory (Updated)"
    course_type = "Mandatory"
    duration = "35 min"
    audience = "Everyone"
    due_date = $yesterday
    content_type = $null
    content_url = $null
} | ConvertTo-Json
$clearedC1 = Invoke-RestMethod -Uri "$baseUrl/learn/courses/$($c1.id)" -Method Put -Headers $adminHeaders -Body $clearBody -ContentType "application/json"
$s16Pass = ($clearedC1.content_type -eq $null) -and ($clearedC1.content_url -eq $null) -and ($clearedC1.duration -eq "35 min")
Report-Scenario "SCENARIO_16" "Admin can explicitly clear resource without affecting other course fields" $s16Pass "content_type=$($clearedC1.content_type), duration=$($clearedC1.duration)"

# ---------------------------------------------------------------------------
# SCENARIO 17: Partial update omitting resource fields retains existing resource
# ---------------------------------------------------------------------------
# Restore resource on C1
$restoreBody = @{
    title = "REC13-C1 Past Due Mandatory (Updated)"
    course_type = "Mandatory"
    duration = "35 min"
    audience = "Everyone"
    due_date = $yesterday
    content_type = "document"
    content_url = "https://company.example.com/materials/c1_restored.pdf"
} | ConvertTo-Json
Invoke-RestMethod -Uri "$baseUrl/learn/courses/$($c1.id)" -Method Put -Headers $adminHeaders -Body $restoreBody -ContentType "application/json" | Out-Null

# Partial update omitting content_type and content_url
$partialBody = @{
    title = "REC13-C1 Renamed Course"
    course_type = "Mandatory"
    duration = "40 min"
    audience = "Everyone"
    due_date = $yesterday
} | ConvertTo-Json
$partialRes = Invoke-RestMethod -Uri "$baseUrl/learn/courses/$($c1.id)" -Method Put -Headers $adminHeaders -Body $partialBody -ContentType "application/json"
$s17Pass = ($partialRes.title -eq "REC13-C1 Renamed Course") -and ($partialRes.content_type -eq "document") -and ($partialRes.content_url -eq "https://company.example.com/materials/c1_restored.pdf")
Report-Scenario "SCENARIO_17" "Partial update omitting resource fields retains existing resource" $s17Pass "Title=$($partialRes.title), Resource=$($partialRes.content_type) -> $($partialRes.content_url)"

# ---------------------------------------------------------------------------
# SCENARIO 18: Invalid URLs and unsafe URI schemes are rejected (HTTP 400)
# ---------------------------------------------------------------------------
$invalidUrls = @(
    "javascript:alert(1)",
    "http://insecure.example.com/doc.pdf",
    "data:text/html,malicious",
    "file:///etc/passwd",
    "https://user:pass@example.com/doc.pdf"
)
$allInvalidRejected = $true
foreach ($badUrl in $invalidUrls) {
    try {
        $badBody = @{
            title = "Invalid URL Course"
            course_type = "Mandatory"
            duration = "30 min"
            audience = "Everyone"
            content_type = "link"
            content_url = $badUrl
        } | ConvertTo-Json
        Invoke-RestMethod -Uri "$baseUrl/learn/courses" -Method Post -Headers $adminHeaders -Body $badBody -ContentType "application/json"
        $allInvalidRejected = $false
        Write-Host "Failed to reject invalid URL: $badUrl" -ForegroundColor Red
    } catch {
        # Expected HTTP 400
    }
}
Report-Scenario "SCENARIO_18" "Unsafe schemes, non-HTTPS, and embedded credentials in URLs are rejected" $allInvalidRejected "Tested: javascript:, http:, data:, file:, user:pass@host"

# ---------------------------------------------------------------------------
# SCENARIO 19: Employee cannot create or modify courses/resources (HTTP 401/403)
# ---------------------------------------------------------------------------
$empTamperBlocked = $false
try {
    $empCourseBody = @{
        title = "Employee Unauthorized Course"
        course_type = "Mandatory"
        duration = "30 min"
        audience = "Everyone"
        content_type = "link"
        content_url = "https://example.com/test"
    } | ConvertTo-Json
    Invoke-RestMethod -Uri "$baseUrl/learn/courses" -Method Post -Headers $empHeaders -Body $empCourseBody -ContentType "application/json"
} catch {
    $empCode = $_.Exception.Response.StatusCode.value__
    if ($empCode -eq 401 -or $empCode -eq 403) {
        $empTamperBlocked = $true
    }
}
Report-Scenario "SCENARIO_19" "Employees cannot create or manage course resources" $empTamperBlocked "Status code: $empCode"

# ---------------------------------------------------------------------------
# SCENARIO 20: Both resource fields must be supplied together (partial resource rejected)
# ---------------------------------------------------------------------------
$typeOnlyRejected = $false
try {
    $typeOnlyBody = @{
        title = "Type Only Course"
        course_type = "Mandatory"
        duration = "30 min"
        audience = "Everyone"
        content_type = "document"
        content_url = ""
    } | ConvertTo-Json
    Invoke-RestMethod -Uri "$baseUrl/learn/courses" -Method Post -Headers $adminHeaders -Body $typeOnlyBody -ContentType "application/json"
} catch {
    if ($_.Exception.Response.StatusCode.value__ -eq 400) {
        $typeOnlyRejected = $true
    }
}
Report-Scenario "SCENARIO_20" "Providing resource type without URL is rejected with HTTP 400" $typeOnlyRejected "HTTP 400 returned"

Write-Host "`n========================================================"
Write-Host "REC-13 ACCEPTANCE TEST SUMMARY"
Write-Host "========================================================"
Write-Host "Total Passed: $passCount / ($passCount + $failCount)"
Write-Host "Total Failed: $failCount"

if ($failCount -gt 0) {
    exit 1
}
exit 0
