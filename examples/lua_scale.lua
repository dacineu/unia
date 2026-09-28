-- Scale reference for `docs/perf-unia-vs-lua.md`. NOT an equivalent: nothing here
-- resembles an act. It is the cost of work a Lua program also pays, so the
-- magnitudes are comparable and the work is not.
local N = 50000
local t = {flow_rate = "0.0", status = "open"}
local s = 0
local a = os.clock()
for i = 1, N do t.flow_rate = "0.0" end
local b = os.clock()
for i = 1, N do
  s = s + 1
  local m = string.format("%s %d", "Valve force-closed", i)
  if m == "" then s = -1 end
end
local c = os.clock()
print(string.format("lua table write      : %8.0f ns/iter", (b-a)/N*1e9))
print(string.format("lua format + arith    : %8.0f ns/iter", (c-b)/N*1e9))
print(string.format("lua combined          : %8.0f ns/iter", (c-a)/N*1e9))
print(string.format("lua throughput        : %8.0f ops/s", N/(c-a)))
