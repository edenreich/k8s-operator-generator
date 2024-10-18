# DogsApi

All URIs are relative to *http://localhost:8080*

| Method | HTTP request | Description |
|------------- | ------------- | -------------|
| [**createDog**](DogsApi.md#createDog) | **POST** /dogs | Create a new dog |
| [**deleteDogById**](DogsApi.md#deleteDogById) | **DELETE** /dogs/{id} | Delete a dog by ID |
| [**getDogById**](DogsApi.md#getDogById) | **GET** /dogs/{id} | Get a dog by ID |
| [**getDogs**](DogsApi.md#getDogs) | **GET** /dogs | Get all dogs |
| [**updateDogById**](DogsApi.md#updateDogById) | **PUT** /dogs/{id} | Update a dog by ID |


<a name="createDog"></a>
# **createDog**
> Dog createDog(Dog)

Create a new dog

### Parameters

|Name | Type | Description  | Notes |
|------------- | ------------- | ------------- | -------------|
| **Dog** | [**Dog**](../Models/Dog.md)|  | |

### Return type

[**Dog**](../Models/Dog.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

<a name="deleteDogById"></a>
# **deleteDogById**
> deleteDogById(id)

Delete a dog by ID

### Parameters

|Name | Type | Description  | Notes |
|------------- | ------------- | ------------- | -------------|
| **id** | **String**|  | [default to null] |

### Return type

null (empty response body)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

<a name="getDogById"></a>
# **getDogById**
> Dog getDogById(id)

Get a dog by ID

### Parameters

|Name | Type | Description  | Notes |
|------------- | ------------- | ------------- | -------------|
| **id** | **String**|  | [default to null] |

### Return type

[**Dog**](../Models/Dog.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

<a name="getDogs"></a>
# **getDogs**
> List getDogs()

Get all dogs

### Parameters
This endpoint does not need any parameter.

### Return type

[**List**](../Models/Dog.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

<a name="updateDogById"></a>
# **updateDogById**
> Dog updateDogById(id, Dog)

Update a dog by ID

### Parameters

|Name | Type | Description  | Notes |
|------------- | ------------- | ------------- | -------------|
| **id** | **String**|  | [default to null] |
| **Dog** | [**Dog**](../Models/Dog.md)|  | |

### Return type

[**Dog**](../Models/Dog.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

