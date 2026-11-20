# Coleta somente leitura para identificar uma combinação exata de hardware/firmware.
# Não coleta números de série de CPU/RAM/placa, UUIDs, endereços de rede nem altera configurações.
$ErrorActionPreference = 'Stop'
$os = Get-CimInstance -ClassName Win32_OperatingSystem
$system = Get-CimInstance -ClassName Win32_ComputerSystem
$board = Get-CimInstance -ClassName Win32_BaseBoard
$bios = Get-CimInstance -ClassName Win32_BIOS
$processors = @(Get-CimInstance -ClassName Win32_Processor | Sort-Object -Property Name)
$processorModels = @($processors | ForEach-Object { ([string]$_.Name).Trim() } | Sort-Object)
$memoryModules = @(
    Get-CimInstance -ClassName Win32_PhysicalMemory | ForEach-Object {
        [pscustomobject][ordered]@{
            DeviceLocator = ([string]$_.DeviceLocator).Trim()
            CapacityBytes = [uint64]($_.Capacity -as [uint64])
            SpeedMhz = [uint32]($_.Speed -as [uint32])
            ConfiguredSpeedMhz = [uint32]($_.ConfiguredClockSpeed -as [uint32])
            SmbiosMemoryType = [uint16]($_.SMBIOSMemoryType -as [uint16])
            Manufacturer = ([string]$_.Manufacturer).Trim()
            PartNumber = ([string]$_.PartNumber).Trim()
        }
    } | Sort-Object -Property DeviceLocator
)

[pscustomobject]@{
    Windows = [ordered]@{
        Edition = $os.Caption
        Version = $os.Version
        Build = $os.BuildNumber
    }
    System = [ordered]@{
        Manufacturer = $system.Manufacturer
        Model = $system.Model
    }
    Motherboard = [ordered]@{
        Manufacturer = $board.Manufacturer
        Product = $board.Product
        Revision = $board.Version
    }
    Firmware = [ordered]@{
        Vendor = $bios.Manufacturer
        Version = $bios.SMBIOSBIOSVersion
        ReleaseDate = $bios.ReleaseDate
    }
    Processor = [ordered]@{
        Models = $processorModels
        PackageCount = $processors.Count
        Cores = [uint32](($processors | Measure-Object -Property NumberOfCores -Sum).Sum)
        LogicalProcessors = [uint32](($processors | Measure-Object -Property NumberOfLogicalProcessors -Sum).Sum)
    }
    Memory = [ordered]@{
        ModuleCount = $memoryModules.Count
        Modules = $memoryModules
    }
} | ConvertTo-Json -Depth 5
