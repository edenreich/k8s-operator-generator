# CatsApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
|------------- | ------------- | -------------|
| [**createCat**](CatsApi.md#createCat) | **POST** /cats | Create a new cat |
| [**deleteCatById**](CatsApi.md#deleteCatById) | **DELETE** /cats/{id} | Delete a cat by ID |
| [**getCatById**](CatsApi.md#getCatById) | **GET** /cats/{id} | Get a cat by ID |
| [**getCats**](CatsApi.md#getCats) | **GET** /cats | Get all cats |
| [**updateCatById**](CatsApi.md#updateCatById) | **PUT** /cats/{id} | Update a cat by ID |


<a name="createCat"></a>
# **createCat**
> Cat createCat(Cat)

Create a new cat

### Parameters

|Name | Type | Description  | Notes |
|------------- | ------------- | ------------- | -------------|
| **Cat** | [**Cat**](../Models/Cat.md)|  | |

### Return type

[**Cat**](../Models/Cat.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

<a name="deleteCatById"></a>
# **deleteCatById**
> deleteCatById(id)

Delete a cat by ID

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

<a name="getCatById"></a>
# **getCatById**
> Cat getCatById(id)

Get a cat by ID

### Parameters

|Name | Type | Description  | Notes |
|------------- | ------------- | ------------- | -------------|
| **id** | **String**|  | [default to null] |

### Return type

[**Cat**](../Models/Cat.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

<a name="getCats"></a>
# **getCats**
> List getCats()

Get all cats

### Parameters
This endpoint does not need any parameter.

### Return type

[**List**](../Models/Cat.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

<a name="updateCatById"></a>
# **updateCatById**
> Cat updateCatById(id, Cat)

Update a cat by ID

### Parameters

|Name | Type | Description  | Notes |
|------------- | ------------- | ------------- | -------------|
| **id** | **String**|  | [default to null] |
| **Cat** | [**Cat**](../Models/Cat.md)|  | |

### Return type

[**Cat**](../Models/Cat.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

