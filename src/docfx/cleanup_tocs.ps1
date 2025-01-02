function Remove-RowsFromFile {
    param (
        [string]$filePath,    # Path to the file
        [int]$startRow,       # Row to start removal (1-based index)
        [int]$numRows         # Number of rows to remove
    )

    # Read the file content as an array of lines
    $fileContent = Get-Content -Path $filePath

    $rowCount = $fileContent.Count
    
    # Validate that startRow and numRows are within valid range
    if ($startRow -lt 1 -or $startRow -gt $rowCount) {
        Write-Host "Error: startRow must be within the valid range of the file ($rowCount)."
        return
    }

    if ($numRows -lt 1) {
        Write-Host "Error: numRows must be a positive integer."
        return
    }

    # Calculate the last row to remove
    $endRow = $startRow + $numRows - 1

    # Ensure we don't exceed the file length
    if ($endRow -gt $fileContent.Count) {
        $endRow = $fileContent.Count
    }

    # Remove the specified rows by selecting all rows except the ones to be removed
    $cleanedContent = $fileContent[0..($startRow-2)] + $fileContent[$endRow..($fileContent.Count-1)]

    # Write the cleaned content back to the file
    $cleanedContent | Set-Content -Path $filePath

    Write-Host "Successfully removed $numRows rows starting at row $startRow."
}

# Example usage:
# Remove rows 2-8 from api/net/toc.yml
Remove-RowsFromFile -filePath "api/net/toc.yml" -startRow 2 -numRows 7