# \DogsApi

All URIs are relative to *http://localhost:8080*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_dog**](DogsApi.md#create_dog) | **POST** /dogs | Create a new dog
[**delete_dog_by_id**](DogsApi.md#delete_dog_by_id) | **DELETE** /dogs/{id} | Delete a dog by ID
[**get_dog_by_id**](DogsApi.md#get_dog_by_id) | **GET** /dogs/{id} | Get a dog by ID
[**get_dogs**](DogsApi.md#get_dogs) | **GET** /dogs | Get all dogs
[**update_dog_by_id**](DogsApi.md#update_dog_by_id) | **PUT** /dogs/{id} | Update a dog by ID



## create_dog

> models::Dog create_dog(dog)
Create a new dog

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**dog** | [**Dog**](Dog.md) |  | [required] |

### Return type

[**models::Dog**](Dog.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_dog_by_id

> delete_dog_by_id(id)
Delete a dog by ID

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** |  | [required] |

### Return type

 (empty response body)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_dog_by_id

> models::Dog get_dog_by_id(id)
Get a dog by ID

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** |  | [required] |

### Return type

[**models::Dog**](Dog.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_dogs

> Vec<models::Dog> get_dogs()
Get all dogs

### Parameters

This endpoint does not need any parameter.

### Return type

[**Vec<models::Dog>**](Dog.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_dog_by_id

> models::Dog update_dog_by_id(id, dog)
Update a dog by ID

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** |  | [required] |
**dog** | [**Dog**](Dog.md) |  | [required] |

### Return type

[**models::Dog**](Dog.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

