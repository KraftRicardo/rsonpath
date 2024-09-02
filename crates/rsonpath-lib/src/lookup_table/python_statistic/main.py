import sys
import os
import pandas as pd
import matplotlib.pyplot as plt


def plot_time(df: pd.DataFrame, save_path: str) -> None:
    """
    Plots the build, CBOR, and JSON times from the dataframe and saves the plot as an image file.
    """
    # Sorting the dataframe based on the 'build' values
    df_sorted = df.sort_values(by=['build'])

    # Plotting build, cbor, json
    plt.figure(figsize=(10, 6))
    plt.plot(df_sorted['name'], df_sorted['build'], marker='o', label='Build')
    plt.plot(df_sorted['name'], df_sorted['cbor_serialize'], marker='o', label='CBOR_SER')
    plt.plot(df_sorted['name'], df_sorted['cbor_deserialize'], marker='o', label='CBOR_DE')
    plt.plot(df_sorted['name'], df_sorted['json_serialize'], marker='o', label='JSON_SER')
    plt.plot(df_sorted['name'], df_sorted['json_deserialize'], marker='o', label='JSON_DE')

    plt.title('Build, CBOR, and JSON Times')
    plt.xlabel('Test Data Sets')
    plt.ylabel('Time in Seconds')
    plt.xticks(rotation=45)
    plt.legend()
    plt.grid(True)
    plt.tight_layout()

    # Adding a comment/note to the plot explaining the labels
    note = ("Build: Build time of the hashtable.\n"
            "CBOR_SER: Build + time to serialize it to CBOR format\n"
            "CBOR_DE: CBOR_SER + deserialize time of the saved file")
    plt.figtext(0.5, -0.15, note, wrap=True, horizontalalignment='center', fontsize=10)

    print(f"Saving statistic to {save_path}")
    plt.savefig(save_path, bbox_inches='tight')


def plot_size(df: pd.DataFrame, save_path: str) -> None:
    """
    Plots the CBOR and JSON sizes from the dataframe and saves the plot as an image file.
    """
    # Sorting the dataframe based on the 'cbor_size' values
    df_sorted = df.sort_values(by=['cbor_size'])

    # Plotting cbor_size and json_size
    plt.figure(figsize=(10, 6))
    plt.plot(df_sorted['name'], df_sorted['cbor_size'],
             marker='o', label='CBOR Size')
    plt.plot(df_sorted['name'], df_sorted['json_size'],
             marker='o', label='JSON Size')

    plt.title('CBOR Size and JSON Size')
    plt.xlabel('Test Data Sets')
    plt.ylabel('Size in Bytes')
    plt.xticks(rotation=45)
    plt.legend()
    plt.grid(True)
    plt.tight_layout()

    print(f"Saving statistic to {save_path}")
    plt.savefig(save_path)


def plot_speed(df: pd.DataFrame, save_path: str) -> None:
    """
    Plots the speed (input size / time) in GB/s for build, CBOR, and JSON operations from the dataframe and saves the 
    plot as an image file.
    """
    df = df.copy()
    # Convert input size from bytes to GB
    df['input_size_gb'] = df['input_size'] / (1024 ** 3)

    # Calculate speeds in GB/s
    df['build_speed'] = df['input_size_gb'] / df['build']
    df['cbor_serialize_speed'] = df['input_size_gb'] / df['cbor_serialize']
    df['cbor_deserialize_speed'] = df['input_size_gb'] / df['cbor_deserialize']
    df['json_serialize_speed'] = df['input_size_gb'] / df['json_serialize']
    df['json_deserialize_speed'] = df['input_size_gb'] / df['json_deserialize']

    # Sorting the dataframe based on the 'build_speed' values
    df_sorted = df.sort_values(by=['build_speed'])

    # Plotting speeds
    plt.figure(figsize=(10, 6))
    plt.plot(df_sorted['name'], df_sorted['build_speed'],
             marker='o', label='Build Speed')
    plt.plot(df_sorted['name'], df_sorted['cbor_serialize_speed'],
             marker='o', label='CBOR Serialize Speed')
    plt.plot(df_sorted['name'], df_sorted['cbor_deserialize_speed'],
             marker='o', label='CBOR Deserialize Speed')
    plt.plot(df_sorted['name'], df_sorted['json_serialize_speed'],
             marker='o', label='JSON Serialize Speed')
    plt.plot(df_sorted['name'], df_sorted['json_deserialize_speed'],
             marker='o', label='JSON Deserialize Speed')

    plt.title('Processing Speed (GB/s)')
    plt.xlabel('Test Data Sets')
    plt.ylabel('Speed in GB/s')
    plt.xticks(rotation=45)
    plt.legend()
    plt.grid(True)
    plt.tight_layout()

    print(f"Saving statistic to {save_path}")
    plt.savefig(save_path)


if __name__ == "__main__":
    # Check if there are any command-line arguments passed
    if len(sys.argv) > 1:
        # Load the CSV file
        file_path = sys.argv[1]
        df = pd.read_csv(file_path)

        # Extract the directory and filename from the file path
        directory, file_name = os.path.split(file_path)
        file_base_name = os.path.splitext(file_name)[0]

        # Plotting the graphs and saving them
        plot_time(df, os.path.join(directory, f"{file_base_name}_time.png"))
        plot_size(df, os.path.join(directory, f"{file_base_name}_size.png"))
        plot_speed(df, os.path.join(directory, f"{file_base_name}_speed.png"))
    else:
        print("No parameters were passed. Please provide the path to the CSV file.")
